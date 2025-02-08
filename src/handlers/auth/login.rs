use warp::Reply;
use warp::{Filter, Rejection, http::StatusCode, reply::Response};
use warp::cookie::cookie; // Import cookie
use crate::models::{Device, Session};
use bcrypt::verify;
use uuid::Uuid;
use std::net::SocketAddr;
use log::{info, error, debug};
use crate::db::users::find_user_by_username;
use crate::db::devices::save_device_to_db;
use crate::db::sessions::save_session_to_db;
use crate::db::devices::find_device_by_ip_mac;
use crate::handlers::auth::{map_validation_errors, LoginData, LoginResponse, LoginSuccessResponse};
use validator::Validate;
use chrono::{Utc, Duration};

pub async fn login_handler(login: LoginData, peer_addr: SocketAddr) -> Result<Response, Rejection> {
    debug!("Received login request: {:?}", login);

    // Валидация данных
    if let Err(errors) = login.validate() {
        error!("Validation errors: {:?}", errors);
        let error_message = map_validation_errors(errors);
        let response = LoginResponse { message: error_message , username: "".to_string()};
        return Ok(warp::reply::with_status(
            warp::reply::json(&response),
            StatusCode::BAD_REQUEST,
        ).into_response());
    }

    let user = match find_user_by_username(&login.username).await {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to find user: {}", e);
            let response = LoginResponse { message: "Failed to find user.".to_string(), username: "".to_string() };
            return Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::UNAUTHORIZED,
            ).into_response());
        }
    };

    match verify(&login.password, &user.password_hash) {
        Ok(valid) => {
            if !valid {
                error!("Invalid password.");
                let response = LoginResponse { message: "Invalid password.".to_string(), username: "".to_string() };
                return Ok(warp::reply::with_status(
                    warp::reply::json(&response),
                    StatusCode::UNAUTHORIZED,
                ).into_response());
            }
        }
        Err(e) => {
            error!("Failed to verify password: {}", e);
            let response = LoginResponse { message: "Failed to verify password.".to_string(), username: "".to_string() };
            return Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::INTERNAL_SERVER_ERROR,
            ).into_response());
        }
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
            ).into_response());
        }
    };

    let session_id = Uuid::new_v4(); // Generate session ID
    let session = Session {
        session_id,
        user_uuid: user.user_uuid,
        device_id: device.device_id,
        expires_at: Some(Utc::now() + Duration::hours(1)),
    };

    if let Err(e) = save_session_to_db(session.clone()).await {
        error!("Failed to save session to database: {}", e);
    }

    info!("User logged in successfully: {}", login.username);
    let response = LoginSuccessResponse {
        message: "User logged in successfully.".to_string(),
        username: login.username.to_string(),
        session_id: session.session_id,
    };

    let mut resp = warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::OK,
    ).into_response();
    
    // **Add the following code:**
    resp.headers_mut().insert(
        "Set-Cookie",
        format!("session_id={}; HttpOnly; Secure; SameSite=Strict; Path=/", session_id).parse().unwrap()
    );

    Ok(resp)
}

pub fn login_route() -> impl Filter<Extract = (Response,), Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("login"))
        .and(warp::body::json())
        .and(warp::addr::remote())
        .and_then(|login: LoginData, addr: Option<SocketAddr>| async move {
            let peer_addr = addr.expect("Failed to get peer address");
            login_handler(login, peer_addr).await
        })
}