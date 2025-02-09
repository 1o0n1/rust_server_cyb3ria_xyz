use crate::models::User;
use log::debug;
use std::error::Error as StdError;
use uuid::Uuid;
use crate::db::connect_to_db;

/// Сохраняет пользователя в базу данных
pub async fn save_user_to_db(user: User) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let client = connect_to_db().await?;  


    debug!("Saving user to database: {}", user.username);

    client.execute(
        "INSERT INTO users (username, password_hash, invitation_code, user_uuid) VALUES ($1, $2, $3, $4)",
        &[&user.username, &user.password_hash, &user.invitation_code, &user.user_uuid],
    )
    .await?;

    Ok(())
}

/// Ищет пользователя по имени
pub async fn find_user_by_username(
    username: &str,
) -> Result<User, Box<dyn StdError + Send + Sync>> {
    let client = connect_to_db().await?;  

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
    let client = connect_to_db().await?;  

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
