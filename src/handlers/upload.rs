use warp::{Filter, Rejection, http::StatusCode};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use std::path::Path;
use serde::{Deserialize, Serialize};
use log::{info, error, debug};
use bytes::Buf; // Import Buf


#[derive(Deserialize, Serialize, Debug, Clone)]
struct UploadResponse {
    message: String,
    filename: Option<String>,
}

pub async fn upload_handler(mut form: warp::multipart::FormData) -> Result<impl warp::Reply, Rejection> {
    debug!("Received file upload request");
    
    while let Some(item) = form.next().await {
        let mut part = match item {
            Ok(part) => part,
            Err(e) => {
                error!("Failed to parse form data: {}", e);
                 let response = UploadResponse { message: "Failed to parse form data".to_string(), filename: None };
                return Ok(warp::reply::with_status(
                    warp::reply::json(&response),
                   StatusCode::BAD_REQUEST,
               ));
            }
        };

        if part.name() == "file" {
            let file_name = match part.filename() {
                Some(file_name) => file_name.to_string(),
                None => {
                     error!("Failed to extract filename");
                     let response = UploadResponse { message: "Failed to extract filename".to_string(), filename: None };
                     return Ok(warp::reply::with_status(
                            warp::reply::json(&response),
                            StatusCode::BAD_REQUEST,
                     ));
                  }
           };
           let file_path = Path::new("/var/www/rust_server_cyb3ria_xyz/uploaded").join(&file_name); // Использование абсолютного пути
            let file_path_str = file_path.to_str().unwrap();
             
            let mut file = match File::create(&file_path).await {
                Ok(file) => file,
                Err(e) => {
                     error!("Failed to create file: {}", e);
                     let response = UploadResponse { message: "Failed to create file.".to_string(), filename: None };
                    return Ok(warp::reply::with_status(
                        warp::reply::json(&response),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                  }
            };
            
            while let Some(chunk) = part.data().await {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(e) => {
                        error!("Failed to read chunk: {}", e);
                         let response = UploadResponse { message: "Failed to read chunk".to_string(), filename: None };
                        return Ok(warp::reply::with_status(
                            warp::reply::json(&response),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        ));
                    }
                };
               
                 if let Err(e) = file.write_all(chunk.chunk()).await {
                    error!("Failed to write to file: {}", e);
                     let response = UploadResponse { message: "Failed to write to file".to_string(), filename: None };
                    return Ok(warp::reply::with_status(
                        warp::reply::json(&response),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                }
            }
             info!("File saved successfully: {}", file_path_str);
             let response = UploadResponse { message: "File uploaded successfully".to_string(), filename: Some(file_name) };
            return Ok(warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::OK,
           ));
        }
    }
    
     let response = UploadResponse { message: "No file found in the form data".to_string(), filename: None };
        Ok(warp::reply::with_status(
            warp::reply::json(&response),
           StatusCode::BAD_REQUEST,
        ))
}

pub fn upload_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("upload"))
        .and(warp::multipart::form())
        .and_then(upload_handler)
}