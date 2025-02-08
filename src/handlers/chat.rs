use warp::ws::{WebSocket, Message};
use futures_util::stream::{StreamExt};
use futures_util::SinkExt;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::sync::Mutex as TokioMutex;
use log::{info, error, debug};
use serde::{Deserialize, Serialize};
use crate::db::messages::save_message_to_db;
use crate::db::send_message_history;
use crate::utils::generate_client_id;
use tokio::time::{Duration as TokioDuration, interval};
use uuid::Uuid;

type Clients = Arc<Mutex<std::collections::HashMap<String, usize>>>;
type Sender = Arc<Mutex<broadcast::Sender<String>>>;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ClientMessage {
    message: String,
    ip: String,
    mac: String,
}

pub async fn client_connection(
    ws: WebSocket,
    clients: Clients,
    sender: Sender,
    user_uuid: Option<String>,
) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let client_ws_sender = Arc::new(TokioMutex::new(client_ws_sender));

    let user_uuid = match user_uuid {
        Some(user_uuid) => user_uuid,
        None => {
            error!("User ID is missing.");
            return;
        }
    };

    let user_uuid_parsed = match Uuid::parse_str(&user_uuid) {
        Ok(uuid) => uuid,
        Err(e) => {
            error!("Failed to parse user ID: {}", e);
            return;
        }
    };

    let user = match crate::db::users::find_user_by_uuid(&user_uuid_parsed).await {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to find user by UUID: {}", e);
            return;
        }
    };
    let username = user.username;

    let client_id = {
        let mut clients = clients.lock().unwrap();
        let client_id = generate_client_id();
        clients.insert(client_id.clone(), 0);
        client_id
    };

    info!("New client connected with ID: {}, username: {}", client_id, username);

    // Отправляем историю сообщений
    if let Err(e) = send_message_history(client_ws_sender.clone()).await {
        error!("Failed to send message history: {}", e);
    }

    let mut rx = sender.lock().unwrap().subscribe();
    let username_clone = username.clone();
    let clients_clone = Arc::clone(&clients);
    let client_id_clone = client_id.clone();

    let ping_interval = TokioDuration::from_secs(30);
    let mut ping_timer = interval(ping_interval);

    let client_ws_sender_task = Arc::clone(&client_ws_sender);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = ping_timer.tick() => {
                    if let Err(e) = client_ws_sender_task.lock().await.send(Message::ping(vec![])).await {
                        error!("Failed to send ping message: {}", e);
                        let mut clients = clients_clone.lock().unwrap();
                        clients.remove(&client_id_clone);
                        info!("Client disconnected with ID: {}, username: {}", client_id_clone, username_clone);
                        break;
                    }
                }
                Ok(message) = rx.recv() => {
                    debug!("Broadcasting message: {}", message);
                    if let Err(e) = client_ws_sender_task.lock().await.send(Message::text(message)).await {
                        error!("Failed to send message: {}", e);
                        let mut clients = clients_clone.lock().unwrap();
                        clients.remove(&client_id_clone);
                        info!("Client disconnected with ID: {}, username: {}", client_id_clone, username_clone);
                        break;
                    }
                }
            }
        }
    });

    while let Some(result) = client_ws_rcv.next().await {
        let msg = if let Ok(msg) = result {
            if msg.is_text() {
                let msg_str = msg.to_str().unwrap().to_owned();
                debug!("Received raw message: {}", msg_str);

                match serde_json::from_str::<ClientMessage>(&msg_str) {
                    Ok(client_message) => {
                        debug!(
                            "Received message from client {}: {}",
                            username, client_message.message
                        );

                        if let Err(e) =
                            save_message_to_db(&client_message.message, user_uuid_parsed).await
                        {
                            error!("Failed to save message to database: {}", e);
                        }

                        let formatted_message = format!("{}: {}", username, client_message.message);
                        formatted_message
                    }
                    Err(e) => {
                        error!("Failed to deserialize message: {}", e);
                        continue;
                    }
                }
            } else if msg.is_close() {
                info!("Client disconnected with ID: {}, username: {}", client_id, username);
                let mut clients = clients.lock().unwrap();
                clients.remove(&client_id);
                break;
            } else {
                continue;
            }
        } else {
            break;
        };

        if let Err(e) = sender.lock().unwrap().send(msg) {
            error!("Failed to send message to broadcast: {}", e);
        }
    }

    if let Err(e) = client_ws_sender.lock().await.close().await {
        error!("Failed to close client connection: {}", e);
    }

    let mut clients = clients.lock().unwrap();
    clients.remove(&client_id);
    info!("Client disconnected with ID: {}, username: {}", client_id, username);
}