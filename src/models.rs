use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub invitation_code: String,
    pub user_uuid: Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Device {
    pub device_id: Uuid,
    pub user_uuid: Uuid,
    pub ip_address: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Session {
    pub session_id: Uuid,
    pub user_uuid: Uuid,
    pub device_id: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Profile {
    pub user_uuid: Uuid,
    pub bio: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProfileResponse {
    pub username: String,
    pub bio: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct File {
    pub file_id: Uuid,
    pub user_uuid: Uuid,
    pub filename: String,
    pub upload_time: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FileInfo {
    pub filename: String,
    pub upload_time: String,
    pub file_id:Uuid
}