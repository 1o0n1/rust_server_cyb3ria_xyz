use tokio_postgres::NoTls;
use std::error::Error as StdError;
use log::{debug};
use crate::models::User;
use uuid::Uuid;

/// Сохраняет пользователя в базу данных
pub async fn save_user_to_db(user: User) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=cyb3ria password=!Abs123 dbname=cyb3ria_db", NoTls)
            .await
            .expect("Failed to connect to database");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    debug!("Saving user to database: {}", user.username);

    client.execute(
        "INSERT INTO users (username, password_hash, invitation_code, user_uuid) VALUES ($1, $2, $3, $4)",
        &[&user.username, &user.password_hash, &user.invitation_code, &user.user_uuid],
    )
    .await?;

    Ok(())
}

/// Ищет пользователя по имени
pub async fn find_user_by_username(username: &str) -> Result<User, Box<dyn StdError + Send + Sync>> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=cyb3ria password=!Abs123 dbname=cyb3ria_db", NoTls)
            .await
            .expect("Failed to connect to database");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    debug!("Finding user in database by username: {}", username);

    let row = client
        .query_one("SELECT username, password_hash, invitation_code, user_uuid FROM users WHERE username = $1", &[&username])
        .await?;

    let user = User {
        username: row.get(0),
        password_hash: row.get(1),
        invitation_code: row.get(2),
        user_uuid: row.get(3),
    };

    Ok(user)
}

/// Ищет пользователя по UUID
pub async fn find_user_by_uuid(user_uuid: &Uuid) -> Result<User, Box<dyn StdError + Send + Sync>> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=cyb3ria password=!Abs123 dbname=cyb3ria_db", NoTls)
            .await
            .expect("Failed to connect to database");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    debug!("Finding user in database by user_uuid: {}", user_uuid);

    let row = client
        .query_one("SELECT username, password_hash, invitation_code, user_uuid FROM users WHERE user_uuid = $1", &[&user_uuid])
        .await?;

    let user = User {
        username: row.get(0),
        password_hash: row.get(1),
        invitation_code: row.get(2),
        user_uuid: row.get(3),
    };

    Ok(user)
}