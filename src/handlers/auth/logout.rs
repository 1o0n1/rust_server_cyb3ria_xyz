// src/handlers/auth/logout.rs

use log::{error, info};
use uuid::Uuid;
use warp::{http::StatusCode, reply::Response, Filter, Rejection, Reply};

pub async fn logout_handler(user_uuid: Uuid) -> Result<Response, Rejection> {
    //  Принимаем user_uuid вместо session_id
    info!("Received logout request for user_uuid: {}", user_uuid);

    // TODO: Удалить сессию из базы данных (реализуйте функцию в db/sessions.rs)
    if let Err(e) = crate::db::sessions::delete_session_by_session_id(&user_uuid).await {
        error!("Failed to delete session: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&"Logout failed"),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response());
    }

    let mut resp = warp::reply::with_status(
        warp::reply::json(&"Logged out successfully"),
        StatusCode::OK,
    )
    .into_response();

    // Очистить куки
    resp.headers_mut().insert(
        "Set-Cookie",
        format!("session_id=; Max-Age=0; Path=/; SameSite=Strict")
            .parse()
            .unwrap(),
    );

    Ok(resp)
}

pub fn logout_route() -> impl Filter<Extract = (Response,), Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("logout"))
        .and(crate::middleware::auth::with_auth()) // Используем middleware для авторизации
        .and_then(|user_uuid: Uuid| async move {
            // Получаем user_uuid из middleware
            logout_handler(user_uuid).await
        })
}
