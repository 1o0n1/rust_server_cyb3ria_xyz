// src/db/files.rs
use crate::models::FileInfo;
use chrono::{DateTime, Utc}; // Добавляем импорт
use log::debug;
use std::error::Error as StdError;
use uuid::Uuid;
use crate::db::connect_to_db;

/// Сохраняет информацию о файле в базу данных
pub async fn save_file_info(
    filename: &str,
    user_uuid: Uuid,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let client = connect_to_db().await?;

    

    debug!(
        "Saving file info to database: filename={}, user_uuid={}",
        filename, user_uuid
    );
    let file_uuid = Uuid::new_v4(); // Add this line

    client.execute(
        "INSERT INTO files (file_id, filename, user_uuid, upload_time) VALUES ($1, $2, $3, NOW())", // Update this line
        &[&file_uuid, &filename, &user_uuid], // Update this line
    )
    .await?;

    Ok(())
}

/// Получает список файлов пользователя из базы данных
/// Получает список файлов пользователя из базы данных
pub async fn get_files_by_user_uuid(
    user_uuid: Uuid,
) -> Result<Vec<FileInfo>, Box<dyn StdError + Send + Sync>> {
    let client = connect_to_db().await?;



    debug!("Getting files for user_uuid: {}", user_uuid);

    let rows = client
        .query(
            "SELECT filename, upload_time, file_id FROM files WHERE user_uuid = $1",
            &[&user_uuid],
        )
        .await?;

    let mut files = Vec::new();
    for row in rows {
        let filename: String = row.get(0);
        let upload_time: DateTime<Utc> = row.get(1);
        let upload_time_string: String = upload_time.to_rfc3339();
        let file_id: Uuid = row.get(2);
        files.push(FileInfo {
            filename,
            upload_time: upload_time_string,
            file_id,
        });
    }

    Ok(files)
}
