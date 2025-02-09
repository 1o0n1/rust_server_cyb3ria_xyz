// src/handlers/upload.rs
use warp::Reply;
use warp::{Filter, Rejection, http::StatusCode, reply::Response};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use std::path::Path;
use serde::{Deserialize, Serialize};
use log::{info, error, debug};
use bytes::Buf;
use uuid::Uuid;
use crate::db::files::save_file_info;
use crate::middleware;
#[derive(Deserialize, Serialize, Debug, Clone)]
struct UploadResponse {
    message: String,
}
pub async fn upload_handler(mut form: warp::multipart::FormData, user_uuid: Uuid) -> Result<Response, Rejection> {
    debug!("Received file upload request");

    while let Some(item) = form.next().await {
        let mut part = match item {
            Ok(part) => part,
            Err(e) => {
                error!("Failed to parse form data: {}", e);
                let response = UploadResponse {
                    message: "Failed to parse form data".to_string(),
                };
                return Ok(warp::reply::with_status(
                    warp::reply::json(&response),
                    StatusCode::BAD_REQUEST,
                ).into_response());
            }
        };

        if part.name() == "file" {
            let file_name = match part.filename() {
                Some(file_name) => file_name.to_string(),
                None => {
                    error!("Failed to extract filename");
                    let response = UploadResponse {
                        message: "Failed to extract filename".to_string(),
                    };
                    return Ok(warp::reply::with_status(
                        warp::reply::json(&response),
                        StatusCode::BAD_REQUEST,
                    ).into_response());
                }
            };
            let file_path = Path::new("/var/www/rust_server_cyb3ria_xyz/uploaded").join(&file_name); // Использование абсолютного пути
            let file_path_str = file_path.to_str().unwrap();

            let mut file = match File::create(&file_path).await {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to create file: {}", e);
                    let response = UploadResponse {
                        message: "Failed to create file.".to_string(),
                    };
                    return Ok(warp::reply::with_status(
                        warp::reply::json(&response),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ).into_response());
                }
            };

            while let Some(chunk) = part.data().await {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(e) => {
                        error!("Failed to read chunk: {}", e);
                        let response = UploadResponse {
                            message: "Failed to read chunk".to_string(),
                        };
                        return Ok(warp::reply::with_status(
                            warp::reply::json(&response),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        ).into_response());
                }
            };

                if let Err(e) = file.write_all(chunk.chunk()).await {
                    error!("Failed to write to file: {}", e);
                    let response = UploadResponse {
                        message: "Failed to write to file".to_string(),
                    };
                    return Ok(warp::reply::with_status(
                        warp::reply::json(&response),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ).into_response());
                }
            }

            info!("File saved successfully: {}", file_path_str);
            
            // Save file info to database
            if let Err(e) = save_file_info(&file_name, user_uuid).await { // Pass user_uuid
                error!("Failed to save file info to database: {}", e);
            }
             let response = UploadResponse {
                message: "Uploaded succesfully!".to_string(),
              };
            return Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::OK,
            ).into_response());

        }
    }

    let response = UploadResponse {
        message: "No file found in the form data".to_string(),
    };
    return Ok(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::BAD_REQUEST,
    ).into_response());

}

pub fn upload_route() -> impl Filter<Extract = (Response,), Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("upload"))
        .and(warp::multipart::form())
        .and(crate::middleware::auth::with_auth()) // Add auth middleware
        .and_then(|form: warp::multipart::FormData, user_uuid: Uuid| async move { // Get user_uuid
            upload_handler(form, user_uuid).await // Pass user_uuid
        })
}