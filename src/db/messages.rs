use bytes::BytesMut;
use chrono::{DateTime, Utc};
use futures_util::SinkExt;
use log::{debug, error};
use std::error::Error as StdError;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio_postgres::types::ToSql;
use tokio_postgres::types::Type;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};

// Обертка для DateTime<Utc>, чтобы обойти "orphan rule"
#[derive(Debug, Copy, Clone)]
struct Timestamp(DateTime<Utc>);

impl ToSql for Timestamp {
    tokio_postgres::types::accepts!(TIMESTAMP, TIMESTAMPTZ);
    tokio_postgres::types::to_sql_checked!();

    fn to_sql(
        &self,
        _type: &Type,
        out: &mut BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn StdError + Sync + Send>> {
        let timestamp = self.0.timestamp();
        (timestamp).to_sql(_type, out).map_err(|e| e.into())
    }
}

/// Экранирует строку для безопасного отображения в HTML
fn escape_html(text: &str) -> String {
    text.replace("&", "&")
        .replace("<", "<")
        .replace(">", ">")
        .replace("\"", "&quot;")
        .replace("'", "'")
}

/// Сохраняет сообщение в базу данных
pub async fn save_message_to_db(
    message: &str,
    user_uuid: Uuid,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=cyb3ria password=!Abs123 dbname='cyb3ria_db'",
        tokio_postgres::NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    debug!(
        "Saving message to database: {}, from user: {}",
        message, user_uuid
    );

    client
        .execute(
            "INSERT INTO messages (message, user_uuid, created_at) VALUES ($1, $2, NOW())",
            &[&message, &user_uuid],
        )
        .await?;

    Ok(())
}

/// Отправляет историю сообщений клиенту
pub async fn send_message_history(
    client_ws_sender: Arc<TokioMutex<futures_util::stream::SplitSink<WebSocket, Message>>>,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=cyb3ria password=!Abs123 dbname=cyb3ria_db",
        tokio_postgres::NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client
        .query(
            "SELECT message, user_uuid FROM messages ORDER BY created_at DESC LIMIT 50",
            &[],
        )
        .await?;

    for row in rows {
        let message: String = row.get(0);
        // Получаем user_uuid, обрабатывая возможность NULL значения
        let user_uuid: Option<Uuid> = row.get(1);

        let formatted_message = match user_uuid {
            Some(uuid) => {
                // Получаем имя пользователя по user_uuid
                let username_result = client
                    .query_one("SELECT username FROM users WHERE user_uuid = $1", &[&uuid])
                    .await;
                match username_result {
                    Ok(row) => {
                        let username: String = row.get(0);
                        let escaped_username = escape_html(&username); // Экранируем имя пользователя
                        format!("{}: {}", escaped_username, message)
                    }
                    Err(e) => {
                        error!("Failed to get username from database: {}", e);
                        format!("Unknown User: {}", message)
                    }
                }
            }
            None => format!("Unknown User: {}", message), // Обработка NULL значения
        };

        if let Err(e) = client_ws_sender
            .lock()
            .await
            .send(Message::text(formatted_message))
            .await
        {
            error!("Failed to send message: {}", e);
        }
    }

    Ok(())
}
