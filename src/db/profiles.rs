// src/db/profiles.rs
use crate::models::Profile;
use log::debug;
use std::error::Error as StdError;
use uuid::Uuid;
use crate::db::connect_to_db;

pub async fn get_profile_by_user_uuid(
    user_uuid: &Uuid,
) -> Result<Option<Profile>, Box<dyn StdError + Send + Sync>> {
    let client = connect_to_db().await?;  


    debug!("Finding profile by user_uuid: {}", user_uuid);

    let row = client
        .query_opt(
            "SELECT user_uuid, bio, avatar FROM profiles WHERE user_uuid = $1",
            &[&user_uuid],
        )
        .await?;

    match row {
        Some(row) => {
            let profile = Profile {
                user_uuid: row.get(0),
                bio: row.get(1),
                avatar: row.get(2),
            };
            Ok(Some(profile))
        }
        None => Ok(None),
    }
}

// Функция для создания профиля (если его нет)
pub async fn create_profile(user_uuid: &Uuid) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let client = connect_to_db().await?;


    debug!("Creating profile for user_uuid: {}", user_uuid);

    client
        .execute(
            "INSERT INTO profiles (user_uuid, bio, avatar) VALUES ($1, NULL, NULL)",
            &[&user_uuid],
        )
        .await?;

    Ok(())
}
