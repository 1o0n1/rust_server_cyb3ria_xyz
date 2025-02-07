use tokio_postgres::NoTls;
use std::error::Error as StdError;
use log::{debug, error};
use crate::models::Session;
use chrono::{DateTime, Utc};
use tokio_postgres::types::{ToSql, Type};
use bytes::BytesMut;
use std::result::Result;
use uuid::Uuid;

// Обертка для DateTime<Utc>, чтобы обойти "orphan rule"
#[derive(Debug)]
struct Timestamp(DateTime<Utc>);

impl ToSql for Timestamp {
    tokio_postgres::types::accepts!(TIMESTAMP, TIMESTAMPTZ);
    tokio_postgres::types::to_sql_checked!();

    fn to_sql(&self, _type: &Type, out: &mut BytesMut) -> Result<tokio_postgres::types::IsNull, Box<dyn StdError + Sync + Send>> {
        let timestamp = self.0.timestamp();
        (timestamp).to_sql(_type, out).map_err(|e| e.into())
    }
}

// Функция для подключения к базе данных
pub async fn connect_to_db() -> Result<tokio_postgres::Client, Box<dyn StdError + Send + Sync>> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=cyb3ria password=!Abs123 dbname=cyb3ria_db", NoTls)
            .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    Ok(client)
}

/// Сохраняет сессию в базу данных
pub async fn save_session_to_db(session: Session) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let client = connect_to_db().await?;

    debug!("Saving session to database: {:?}", session);

    let expires_at = session.expires_at.map(Timestamp);

    client.execute(
        "INSERT INTO sessions (session_id, user_uuid, device_id, expires_at) VALUES ($1, $2, $3, $4)",
        &[&session.session_id, &session.user_uuid, &session.device_id, &expires_at],
    )
    .await?;

    Ok(())
}

/// Ищет сессию по session_id
pub async fn find_session_by_session_id(session_id: &Uuid) -> Result<Option<Session>, Box<dyn StdError + Send + Sync>> {
    let client = connect_to_db().await?;

    debug!("Finding session in database by session_id: {}", session_id);

    let row = client
        .query_opt("SELECT session_id, user_uuid, device_id, expires_at FROM sessions WHERE session_id = $1", &[&session_id])
        .await?;

    if let Some(row) = row {
        let expires_at: Option<i64> = row.get(3);
        let expires_at_datetime: Option<DateTime<Utc>> = match expires_at {
            Some(ts) => {
                match DateTime::<Utc>::from_timestamp(ts, 0) {
                    Some(dt) => Some(dt),
                    None => {
                        error!("Invalid timestamp: {}", ts);
                        None // Or handle the error as you see fit, perhaps return an error
                    }
                }
            }
            None => None,
        };

        let session = Session {
            session_id: row.get(0),
            user_uuid: row.get(1),
            device_id: row.get(2),
            expires_at: expires_at_datetime,
        };
        Ok(Some(session))
    } else {
        Ok(None)
    }
}