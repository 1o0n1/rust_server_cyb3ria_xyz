use tokio_postgres::NoTls;
use std::error::Error as StdError;
use log::{error, debug};
use crate::models::Device;
use std::net::IpAddr;

/// Сохраняет устройство в базу данных
pub async fn save_device_to_db(device: Device) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=cyb3ria password=!Abs123 dbname=cyb3ria_db", NoTls)
            .await
            .expect("Failed to connect to database");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    debug!("Saving device to database: {:?}", device);

    // Преобразуем IP-адрес в тип IpAddr
    let ip_address: IpAddr = device.ip_address.parse().map_err(|e| {
        error!("Failed to parse IP address: {}", e);
        e
    })?;

    client.execute(
        "INSERT INTO devices (device_id, user_uuid, ip_address) VALUES ($1, $2, $3)",
        &[&device.device_id, &device.user_uuid, &ip_address],
    )
    .await?;

    Ok(())
}

/// Ищет устройство по IP-адресу
pub async fn find_device_by_ip_mac(ip_address: &str) -> Result<Option<Device>, Box<dyn StdError + Send + Sync>> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=cyb3ria password=!Abs123 dbname=cyb3ria_db", NoTls)
            .await
            .expect("Failed to connect to database");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    debug!("Finding device by IP: {}", ip_address);

    // Преобразуем IP-адрес в тип IpAddr
    let ip_addr: IpAddr = ip_address.parse().map_err(|e| {
        error!("Failed to parse IP address: {}", e);
        e
    })?;


    let row = client.query_opt(
            "SELECT device_id, user_uuid, ip_address FROM devices WHERE ip_address = $1",
            &[&ip_addr],
        )
        .await?;
    

    if let Some(row) = row {
        let device = Device {
            device_id: row.get(0),
            user_uuid: row.get(1),
            ip_address: row.get::<_, IpAddr>(2).to_string(),  // Преобразуем обратно в строку
           
        };
        Ok(Some(device))
    } else {
        Ok(None)
    }
}