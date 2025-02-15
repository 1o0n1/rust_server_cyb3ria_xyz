// src/handlers/profile.rs

use crate::db::profiles::{create_profile, get_profile_by_user_uuid, update_profile};
use crate::db::users::find_user_by_uuid;
use crate::models::{ProfileResponse, UpdateProfileRequest, UpdateProfileResponse};
use log::{debug, error};
use uuid::Uuid;
use warp::Reply;
use warp::{http::StatusCode, reply::Response, Filter, Rejection, reply::json};
use chrono::Utc;

pub async fn profile_handler(user_uuid: Uuid) -> Result<Response, Rejection> {
    debug!("Received profile request for user_uuid: {}", user_uuid);

    // Сначала пробуем получить профиль из БД
    let profile = match get_profile_by_user_uuid(&user_uuid).await {
        Ok(Some(profile)) => profile,
        Ok(None) => {
            // Если профиля нет, создаем его
            if let Err(e) = create_profile(&user_uuid).await {
                error!("Failed to create profile: {}", e);
                return Ok(warp::reply::with_status(
                    warp::reply::json(&"Failed to create profile"),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response());
            }
            // Получаем созданный профиль
            get_profile_by_user_uuid(&user_uuid).await.unwrap().unwrap()
        }
        Err(e) => {
            error!("Failed to get profile: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&"Failed to get profile"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    };

    // Получаем имя пользователя и дату регистрации
    let user = find_user_by_uuid(&user_uuid).await.map_err(|e| {
        error!("Failed to get user: {}", e);
        warp::reject::reject()
    })?;

    let profile_response = ProfileResponse {
        username: user.username,
        bio: profile.bio,
        avatar: profile.avatar,
        profile_banner: profile.profile_banner,
        registration_date: Utc::now(),  //  Заменить на реальную дату регистрации
        online_status: false, // Реализуем позже
        storage_access: profile.storage_access,
    };

    Ok(
        warp::reply::with_status(warp::reply::json(&profile_response), StatusCode::OK)
            .into_response(),
    )
}


pub async fn update_profile_handler(
    user_uuid: Uuid,
    request: UpdateProfileRequest,
) -> Result<Response, Rejection> {
    debug!(
        "Received update profile request for user_uuid: {}, request: {:?}",
        user_uuid, request
    );

    println!("Received bio: {:?}", request.bio);

    if let Err(e) = update_profile(&user_uuid, request).await {
        error!("Failed to update profile: {}", e);
        let response = UpdateProfileResponse {
            message: "Failed to update profile".to_string(),
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&response),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response());
    }

    let response = UpdateProfileResponse {
        message: "Profile updated successfully".to_string(),
    };
    Ok(
        warp::reply::with_status(warp::reply::json(&response), StatusCode::OK).into_response(),
    )
}

pub fn profile_route() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let get_profile = warp::path("api")
        .and(warp::path("profile"))
        .and(warp::get())
        .and(warp::path::param::<Uuid>())
        .and_then(|user_uuid: Uuid| async move {
            let result = profile_handler(user_uuid).await;
            match result {
                Ok(response) => Ok(response),
                Err(rejection) => Err(rejection),
            }
        });

    let update_profile = warp::path("api")
        .and(warp::path("profile"))
        .and(warp::put())
        .and(crate::middleware::auth::with_auth())
        .and(warp::body::json())
        .and_then(|user_uuid: Uuid, request: UpdateProfileRequest| async move {
            update_profile_handler(user_uuid, request).await
        });

    get_profile.or(update_profile)
}