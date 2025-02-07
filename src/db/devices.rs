use std::net::IpAddr;
use std::error::Error as StdError;
use log::{error, debug};
use crate::models::Device;
use crate::db::sessions::connect_to_db; // Импортируем функцию

/// Сохраняет устройство в базу данных
pub async fn save_device_to_db(device: Device) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let client = connect_to_db().await?;

    debug!("Saving device to database: {:?}", device);

    client.execute(
        "INSERT INTO devices (device_id, user_uuid, ip_address) VALUES ($1, $2, $3)",
        &[&device.device_id, &device.user_uuid, &device.ip_address.parse::<IpAddr>().unwrap()],
    )
    .await?;

    Ok(())
}

/// Ищет устройство по IP-адресу
pub async fn find_device_by_ip_mac(ip_address: &str) -> Result<Option<Device>, Box<dyn StdError + Send + Sync>> {
    use std::net::IpAddr;
    let client = connect_to_db().await?;

    debug!("Finding device by IP: {}", ip_address);

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
            ip_address: row.get::<_, IpAddr>(2).to_string(),
           
        };
        Ok(Some(device))
    } else {
        Ok(None)
    }
}