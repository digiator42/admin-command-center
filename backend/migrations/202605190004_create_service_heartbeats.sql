-- rolling health history
CREATE TABLE IF NOT EXISTS service_heartbeats (
    id SERIAL PRIMARY KEY,
    service_id INT NOT NULL REFERENCES monitored_services(id) ON DELETE CASCADE,
    latency_ms INT NOT NULL,
    http_status_code SMALLINT NOT NULL,
    status_msg VARCHAR(50) NOT NULL,
    error_log TEXT,
    checked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
