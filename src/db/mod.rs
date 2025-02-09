pub mod devices;
pub mod files;
pub mod messages;
pub mod profiles;
pub mod sessions;
pub mod users;
pub use messages::send_message_history;

use tokio_postgres::NoTls;
use std::error::Error as StdError;
pub use log::error;
pub use std::env; // Добавляем импорт

pub async fn connect_to_db() -> Result<tokio_postgres::Client, Box<dyn StdError + Send + Sync>> {
    // Получаем строку подключения из переменной окружения
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set"); //  Если переменной нет, будет паника

    let (client, connection) =
        tokio_postgres::connect(&database_url, NoTls) // Используем database_url
            .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    Ok(client)
}