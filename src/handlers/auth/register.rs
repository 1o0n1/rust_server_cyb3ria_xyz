// src/handlers/auth/register.rs

use crate::db::users::save_user_to_db;
use crate::handlers::auth::{map_validation_errors, RegistrationData, RegistrationResponse};
use crate::models::User;
use bcrypt::{hash, DEFAULT_COST};
use log::{debug, error, info};
use std::net::SocketAddr;
use uuid::Uuid;
use validator::Validate;
use warp::Reply;
use warp::{http::StatusCode, reply::Response, Filter, Rejection};

pub async fn register_handler(
    registration: RegistrationData,
    _peer_addr: SocketAddr,
) -> Result<Response, Rejection> {
    debug!("Received registration request: {:?}", registration);

    // Валидация данных
    if let Err(errors) = registration.validate() {
        error!("Validation errors: {:?}", errors);
        let error_message = map_validation_errors(errors);
        let response = RegistrationResponse {
            message: error_message,
        };
        return Ok(
            warp::reply::with_status(warp::reply::json(&response), StatusCode::BAD_REQUEST)
                .into_response(),
        );
    }

    if registration.password != registration.repeat_password {
        error!("Passwords do not match.");
        let response = RegistrationResponse {
            message: "Passwords do not match.".to_string(),
        };
        return Ok(
            warp::reply::with_status(warp::reply::json(&response), StatusCode::BAD_REQUEST)
                .into_response(),
        );
    }

    let password_hash = match hash(registration.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash password: {}", e);
            let response = RegistrationResponse {
                message: "Failed to hash password.".to_string(),
            };
            return Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    };

    let user_uuid = Uuid::new_v4();

    let user = User {
        username: registration.username,
        password_hash,
        invitation_code: registration.invitation_code,
        user_uuid,
    };

    match save_user_to_db(user).await {
        Ok(_) => {
            info!("User registered successfully. Redirecting to login page.");
            //  Редирект на страницу логина.  Здесь вместо JSON, простой редирект
            let resp = warp::reply::with_status(warp::reply::html(""), StatusCode::FOUND)
                .into_response(); //  Использовали HTML для редиректа.

             let mut resp =
                 warp::reply::with_status(warp::reply::json(&""), StatusCode::FOUND).into_response();

             resp.headers_mut().insert(
                 "Location",
                 "/static/login.html".parse().unwrap(),
             );
             Ok(resp)
        }
        Err(e) => {
            error!("Failed to save user to database: {}", e);
            let response = RegistrationResponse {
                message: "Failed to save user to database.".to_string(),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response())
        }
    }
}

pub fn register_route() -> impl Filter<Extract = (Response,), Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("register"))
        .and(warp::body::json())
        .and(warp::addr::remote())
        .and_then(
            |registration: RegistrationData, addr: Option<SocketAddr>| async move {
                let peer_addr = addr.expect("Failed to get peer address");
                register_handler(registration, peer_addr).await
            },
        )
}