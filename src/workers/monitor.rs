use crate::models::{monitored_services, service_heartbeats};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::sync::Arc;

pub async fn start_service_monitor_loop(db: Arc<DatabaseConnection>) {
    // Create a persistent HTTP client using `reqwest`
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5)) // Don't hang forever if it crashes
        .build()
        .unwrap();

    loop {
        // Fetch all active services from `monitored_services` table
        if let Ok(services) = monitored_services::Entity::find().all(db.as_ref()).await {
            for service in services {
                let start_time = std::time::Instant::now();

                // Ping the target endpoint asynchronously
                let response = client.get(&service.target_url).send().await;
                let latency = start_time.elapsed().as_millis() as i32;

                // Evaluate the result status
                let (status_str, status_code, err_msg) = match response {
                    Ok(res) => {
                        let code = res.status().as_u16();
                        if code == 200 {
                            ("UP", code, None)
                        } else {
                            ("DEGRADED", code, Some(format!("Returned status {}", code)))
                        }
                    }
                    Err(err) => ("DOWN", 0, Some(err.to_string())),
                };

                // Update the state inside the database
                // - Write a row to `service_heartbeats` logging the latency and timestamp
                // - Update the `current_status` in `monitored_services` if it changed
                log_heartbeat(
                    &db,
                    service.id,
                    latency,
                    status_code as i16,
                    &status_str,
                    err_msg,
                )
                .await;
            }
        }

        // Sleep for the desired frequency pool interval before checking again
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}

/// Helper function to log heartbeat history and update service status
async fn log_heartbeat(
    db: &DatabaseConnection,
    service_id: i32,
    latency_ms: i32,
    status_code: i16,
    status_str: &str,
    error_log: Option<String>,
) {
    // Create a new heartbeat log record
    let heartbeat_record = service_heartbeats::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        service_id: Set(service_id),
        latency_ms: Set(latency_ms),
        http_status_code: Set(status_code),
        status_msg: Set(status_str.to_string()),
        error_log: Set(error_log),
        checked_at: Set(None), // DB defaults to CURRENT_TIMESTAMP automatically
    };

    // Save the log record to the database
    if let Err(err) = heartbeat_record.insert(db).await {
        eprintln!(
            "[MONITOR ERROR] Failed to write heartbeat ledger log: {}",
            err
        );
    }

    // Update the master inventory service status record with current state
    if let Ok(Some(service)) = monitored_services::Entity::find_by_id(service_id)
        .one(db)
        .await
    {
        let mut active_service: monitored_services::ActiveModel = service.into();
        active_service.current_status = Set(status_str.to_string());

        if let Err(err) = active_service.update(db).await {
            eprintln!(
                "[MONITOR ERROR] Failed to update service status cache: {}",
                err
            );
        }
    }
}
