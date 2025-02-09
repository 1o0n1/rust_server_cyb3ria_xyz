// src/middleware/auth.rs
use crate::db::sessions::find_session_by_session_id;
use log::{debug, error};
use uuid::Uuid;
use warp::{http::StatusCode, Filter, Rejection};

pub fn with_auth() -> impl Filter<Extract = (Uuid,), Error = Rejection> + Clone {
    warp::cookie("session_id").and_then(|session_id: String| async move {
        debug!("with_auth: session_id from cookie: {}", session_id);
        let session_uuid = match Uuid::parse_str(&session_id) {
            Ok(uuid) => {
                debug!("with_auth: Parsed session_uuid: {}", uuid);
                uuid
            }
            Err(e) => {
                error!("with_auth: Failed to parse session_id: {}", e);
                return Err(warp::reject::reject());
            }
        };

        match find_session_by_session_id(&session_uuid).await {
            Ok(Some(session)) => {
                debug!("with_auth: Session found in DB: {:?}", session);
                Ok(session.user_uuid) // Return user_uuid
            }
            Ok(None) => {
                error!("with_auth: Session not found in DB");
                Err(warp::reject::reject())
            }
            Err(e) => {
                error!("with_auth: Error finding session in DB: {}", e);
                Err(warp::reject::reject())
            }
        }
    })
}
