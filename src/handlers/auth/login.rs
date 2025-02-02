use warp::{Filter, Rejection, http::StatusCode};
use crate::models::{Device, Session};
use bcrypt::verify;
use uuid::Uuid;
use std::net::SocketAddr;
use log::{info, error, debug};
use crate::db::users::find_user_by_username;
use crate::db::devices::save_device_to_db;
use crate::db::sessions::save_session_to_db;
use crate::db::devices::find_device_by_ip_mac;
use crate::handlers::auth::{map_validation_errors, LoginData, LoginResponse};
use validator::Validate;
use chrono::{Utc, Duration};

pub async fn login_handler(login: LoginData, peer_addr: SocketAddr) -> Result<impl warp::Reply, Rejection> {
    debug!("Received login request: {:?}", login);

    // Валидация данных
    if let Err(errors) = login.validate() {
        error!("Validation errors: {:?}", errors);
        let error_message = map_validation_errors(errors);
        let response = LoginResponse { message: error_message , username: "".to_string()};
        return Ok(warp::reply::with_status(
            warp::reply::json(&response),
            StatusCode::BAD_REQUEST,
        ));
    }

    let user = match find_user_by_username(&login.username).await {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to find user: {}", e);
            let response = LoginResponse { message: "Failed to find user.".to_string(), username: "".to_string() };
            return Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::UNAUTHORIZED,
            ));
        }
    };

    if !verify(&login.password, &user.password_hash).unwrap_or(false) {
        error!("Invalid password.");
        let response = LoginResponse { message: "Invalid password.".to_string(), username: "".to_string() };
        return Ok(warp::reply::with_status(
            warp::reply::json(&response),
            StatusCode::UNAUTHORIZED,
        ));
    }

    let device = match find_device_by_ip_mac(&peer_addr.ip().to_string()).await {
        Ok(Some(device)) => device,
        Ok(None) => {
            let device = Device {
                device_id: Uuid::new_v4(),
                user_uuid: user.user_uuid,
                ip_address: peer_addr.ip().to_string(),
            };
            if let Err(e) = save_device_to_db(device.clone()).await {
                error!("Failed to save device to database: {}", e);
            }
            device
        },
        Err(e) => {
            error!("Failed to find device: {}", e);
            let response = LoginResponse { message: "Failed to find device.".to_string(), username: "".to_string() };
            return Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };
    
    let session = Session {
        session_id: Uuid::new_v4(),
        user_uuid: user.user_uuid,
        device_id: device.device_id,
        expires_at: Some(Utc::now() + Duration::hours(1)),
    };

    if let Err(e) = save_session_to_db(session.clone()).await {
        error!("Failed to save session to database: {}", e);
    }

    info!("User logged in successfully: {}", login.username);
    let response = LoginResponse {
        message: "User logged in successfully.".to_string(),
        username: login.username.to_string()
    };
    
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::OK,
    ))
}

pub fn login_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("login"))
        .and(warp::body::json())
        .and(warp::addr::remote())
        .and_then(|login: LoginData, addr: Option<SocketAddr>| async move {
            let peer_addr = addr.expect("Failed to get peer address");
            login_handler(login, peer_addr).await
        })
}