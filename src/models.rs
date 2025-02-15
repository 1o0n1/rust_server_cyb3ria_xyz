use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tokio_postgres::types::{FromSql, Type, IsNull};
use std::error::Error as StdError;
use std::fmt;



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
    pub profile_banner: Option<String>,
    pub storage_access: StorageAccess,
    pub allowed_viewers: Vec<Uuid>,  // Список UUID пользователей с доступом
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProfileResponse {
    pub username: String,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub profile_banner: Option<String>,
    pub registration_date: DateTime<Utc>,
    pub online_status: bool, //  Реализуем позже
    pub storage_access: StorageAccess,
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
    pub file_id: Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpdateProfileRequest {
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub profile_banner: Option<String>,
    pub storage_access: StorageAccess,
    pub allowed_viewers: Option<Vec<Uuid>>,  // Список UUID пользователей с доступом
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpdateProfileResponse {
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum StorageAccess {
    Private,        // Только пользователь
    Public,         // Всем
    SpecificUsers,  // Только указанным пользователям
}


impl fmt::Display for StorageAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageAccess::Private => write!(f, "Private"),
            StorageAccess::Public => write!(f, "Public"),
            StorageAccess::SpecificUsers => write!(f, "SpecificUsers"),
        }
    }
}

impl<'a> FromSql<'a> for StorageAccess {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn StdError + Sync + Send>> {
         let s = String::from_utf8(raw.to_vec())?;
            match s.as_str() {
                "Private" => Ok(StorageAccess::Private),
                "Public" => Ok(StorageAccess::Public),
                "SpecificUsers" => Ok(StorageAccess::SpecificUsers),
                _ => Err(format!("Invalid storage access value: {}", s).into()),
            }
    }

    fn accepts(ty: &Type) -> bool {
        match *ty {
            Type::TEXT | Type::VARCHAR | Type::NAME  => true,
            _ => false,
        }
    }
}
