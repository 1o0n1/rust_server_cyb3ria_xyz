// src/handlers/profile.rs
use warp::{Reply, Filter, Rejection, http::StatusCode, reply::Response};
use crate::models::{ProfileResponse};
use crate::db::users::find_user_by_uuid;
use crate::db::profiles::{get_profile_by_user_uuid, create_profile};
use uuid::Uuid;
use log::{error, debug};

pub async fn profile_handler(user_uuid: Uuid) -> Result<Response, Rejection> {  // Получаем user_uuid из middleware
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
                ).into_response());
            }
            // Получаем созданный профиль
            get_profile_by_user_uuid(&user_uuid).await.unwrap().unwrap()
        }
        Err(e) => {
            error!("Failed to get profile: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&"Failed to get profile"),
                StatusCode::INTERNAL_SERVER_ERROR,
            ).into_response());
        }
    };

    // Получаем имя пользователя
    let user = find_user_by_uuid(&user_uuid).await.map_err(|e| {
         error!("Failed to get user: {}", e);
        warp::reject::reject()
    })?;

    let profile_response = ProfileResponse {
        username: user.username,
        bio: profile.bio,
        avatar: profile.avatar,
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&profile_response),
        StatusCode::OK,
    ).into_response())
}

pub fn profile_route() -> impl Filter<Extract = (Response,), Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("profile"))
        .and(crate::middleware::auth::with_auth()) // Используем middleware для авторизации
        .and_then(|user_uuid: Uuid| async move {  // Получаем user_uuid из middleware
            profile_handler(user_uuid).await
        })
}