// src/handlers/files.rs
use crate::db::files::get_files_by_user_uuid;
use crate::models::FileInfo;
use log::{debug, error};
use uuid::Uuid;
use warp::{http::StatusCode, reply::Json, reply::Response, Filter, Rejection};
pub async fn get_files_handler(user_uuid: Uuid) -> Result<Json, Rejection> {
    debug!("Received request for files for user_uuid: {}", user_uuid);

    match get_files_by_user_uuid(user_uuid).await {
        Ok(files) => {
            let file_info: Vec<FileInfo> = files.into_iter().collect();
            Ok(warp::reply::json(&file_info))
        }
        Err(e) => {
            error!("Failed to get files: {}", e);
            Err(warp::reject::reject())
        }
    }
}

pub fn files_route() -> impl Filter<Extract = (Json,), Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("files"))
        .and(crate::middleware::auth::with_auth())
        .and_then(|user_uuid: Uuid| async move { get_files_handler(user_uuid).await })
}
