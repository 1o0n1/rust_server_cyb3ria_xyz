mod utils;
mod models;
mod handlers;
mod db;
mod middleware;

use uuid::Uuid;
use handlers::profile::profile_route;
use handlers::auth::logout::logout_route;

use warp::Filter;
use dotenv::dotenv;
use log::info;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::broadcast;
use handlers::auth::{register::register_route, login::login_route};
use handlers::chat::client_connection;
use handlers::upload::upload_route;


type Clients = Arc<Mutex<std::collections::HashMap<String, usize>>>;
type Sender = Arc<Mutex<broadcast::Sender<String>>>;

#[tokio::main]
async fn main() {
    // Загрузка переменных окружения из .env файла
    dotenv().ok();

    // Инициализация логирования
    env_logger::init();

    // Логирование начала работы сервера
    info!("Initializing server ...");

    let clients: Clients = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let sender: Sender = Arc::new(Mutex::new(broadcast::channel(100).0));
    let clients_clone = Arc::clone(&clients);
    let sender_clone = Arc::clone(&sender);

    let chat_route = warp::path("api")
    .and(warp::path("ws"))
    .and(warp::ws())
    .and(warp::addr::remote())
    .and(crate::middleware::auth::with_auth()) // Используем middleware для авторизации
    .map(
        move |ws: warp::ws::Ws, _addr: Option<std::net::SocketAddr>, user_uuid: Uuid| { //user_uuid получаем из middleware
            let clients_clone = Arc::clone(&clients_clone);
            let sender_clone = Arc::clone(&sender_clone);
            //let session_id = params.get("session_id").map(|s| s.to_string());  //session_id больше не нужен
            ws.on_upgrade(move |socket| {
                client_connection(socket, clients_clone, sender_clone, Some(user_uuid.to_string()))  //Передаём user_uuid в client_connection
            })
        },
    )
    .boxed();

    let register_route = register_route().boxed();
    let login_route = login_route().boxed();
    let upload_route = upload_route().boxed();
    let profile_route = profile_route().boxed(); // Добавляем роут профиля
    let logout_route = logout_route().boxed();

    let routes = chat_route
    .or(register_route)
    .or(login_route)
    .or(upload_route)
    .or(profile_route) // Регистрируем роут профиля
    .or(logout_route);  // Регистрируем роут logout

    

    info!("Starting server on 127.0.0.1:8081");
    warp::serve(routes).run(([127, 0, 0, 1], 8081)).await;
}