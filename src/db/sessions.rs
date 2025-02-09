use crate::models::Session;
use bytes::BytesMut;
use chrono::{DateTime, Utc};
use log::{debug, error};
use std::error::Error as StdError;
use std::result::Result;
use tokio_postgres::types::{ToSql, Type};
use tokio_postgres::NoTls;
use uuid::Uuid;

// Обертка для DateTime<Utc>, чтобы обойти "orphan rule"
#[derive(Debug, Copy, Clone)]
struct Timestamp(DateTime<Utc>);

impl ToSql for Timestamp {
    tokio_postgres::types::accepts!(TIMESTAMP, TIMESTAMPTZ);
    tokio_postgres::types::to_sql_checked!();

    fn to_sql(
        &self,
        _type: &Type,
        out: &mut BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn StdError + Sync + Send>> {
        let timestamp = self.0.timestamp();
        (timestamp).to_sql(_type, out)
    }
}

impl<'a> tokio_postgres::types::FromSql<'a> for Timestamp {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn StdError + Sync + Send>> {
        let timestamp: i64 = tokio_postgres::types::FromSql::from_sql(ty, raw)?;
        let datetime = DateTime::<Utc>::from_timestamp(timestamp, 0).ok_or("invalid timestamp")?;
        Ok(Timestamp(datetime))
    }

    tokio_postgres::types::accepts!(TIMESTAMP, TIMESTAMPTZ);
}

// Функция для подключения к базе данных
pub async fn connect_to_db() -> Result<tokio_postgres::Client, Box<dyn StdError + Send + Sync>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=cyb3ria password=!Abs123 dbname=cyb3ria_db",
        NoTls,
    )
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
pub async fn find_session_by_session_id(
    session_id: &Uuid,
) -> Result<Option<Session>, Box<dyn StdError + Send + Sync>> {
    let client = connect_to_db().await?;

    debug!("Finding session in database by session_id: {}", session_id);

    let row = client
        .query_opt("SELECT session_id, user_uuid, device_id, expires_at FROM sessions WHERE session_id = $1", &[&session_id])
        .await?;

    if let Some(row) = row {
        let expires_at: Option<Timestamp> = row.get(3); // Получаем expires_at как Option<Timestamp>
        let expires_at_datetime: Option<DateTime<Utc>> = expires_at.map(|ts| ts.0); // Преобразуем Timestamp в DateTime<Utc>

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
/// Удаляет сессию из базы данных по session_id
pub async fn delete_session_by_session_id(
    session_id: &Uuid,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=cyb3ria password=!Abs123 dbname=cyb3ria_db",
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    debug!(
        "Deleting session from database with session_id: {}",
        session_id
    );

    client
        .execute("DELETE FROM sessions WHERE session_id = $1", &[&session_id])
        .await?;

    Ok(())
}
