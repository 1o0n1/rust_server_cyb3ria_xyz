// src/db/profiles.rs
use crate::models::{Profile, UpdateProfileRequest, StorageAccess};
use log::debug;
use std::error::Error as StdError;
use uuid::Uuid;
use crate::db::connect_to_db;
use tokio_postgres::types::ToSql;

pub async fn get_profile_by_user_uuid(
    user_uuid: &Uuid,
) -> Result<Option<Profile>, Box<dyn StdError + Send + Sync>> {
    let client = connect_to_db().await?;

    debug!("Finding profile by user_uuid: {}", user_uuid);

    let row = client
        .query_opt(
            "SELECT user_uuid, bio, avatar, profile_banner, storage_access, allowed_viewers FROM profiles WHERE user_uuid = $1",
            &[&user_uuid],
        )
        .await?;

    match row {
        Some(row) => {
            let storage_access: StorageAccess = row.get(4);
            let profile = Profile {
                user_uuid: row.get(0),
                bio: row.get(1),
                avatar: row.get(2),
                profile_banner: row.get(3),
                storage_access: storage_access,
                allowed_viewers: row.get(5), // Убедитесь, что тип данных соответствует массиву UUID
            };
            Ok(Some(profile))
        }
        None => Ok(None),
    }
}

pub async fn create_profile(user_uuid: &Uuid) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let client = connect_to_db().await?;

    debug!("Creating profile for user_uuid: {}", user_uuid);

    client
        .execute(
            "INSERT INTO profiles (user_uuid, bio, avatar, profile_banner, storage_access, allowed_viewers) VALUES ($1, NULL, NULL, NULL, 'Private', ARRAY[]::uuid[])",
            &[&user_uuid],
        )
        .await?;

    Ok(())
}

pub async fn update_profile(
    user_uuid: &Uuid,
    request: UpdateProfileRequest,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let client = connect_to_db().await?;

    debug!("Updating profile for user_uuid: {}, request: {:?}", user_uuid, request);

    let mut query = "UPDATE profiles SET".to_string();
    let mut params: Vec<&(dyn ToSql + Sync)> = Vec::new();
    let mut param_index = 1;

    
    query.push_str(&format!(" bio = ${},", param_index));
    match &request.bio {
        Some(bio) => params.push(bio),
        None => params.push(&None::<String>),
    };
    param_index += 1;


    query.push_str(&format!(" avatar = ${},", param_index));
    match &request.avatar {
        Some(avatar) => params.push(avatar),
        None => params.push(&None::<String>),
    };
    param_index += 1;


    query.push_str(&format!(" profile_banner = ${},", param_index));
    match &request.profile_banner {
        Some(profile_banner) => params.push(profile_banner),
        None => params.push(&None::<String>),
    };
    param_index += 1;

    query.push_str(&format!(" storage_access = ${},", param_index));
    let storage_access_string = request.storage_access.to_string();
    params.push(&storage_access_string);
    param_index += 1;

    if let Some(allowed_viewers) = &request.allowed_viewers {
        query.push_str(&format!(" allowed_viewers = ${},", param_index));
        params.push(allowed_viewers);
        param_index += 1;
    } else {
        query.push_str(" allowed_viewers = ARRAY[]::uuid[],");
    }

    if query.ends_with(",") {
        query.pop();
    }

    query.push_str(&format!(" WHERE user_uuid = ${}", param_index));
    params.push(&user_uuid);

    debug!("Executing query: {}, params: {:?}", query, params);

    client.execute(&query, &params).await?;

    Ok(())
}