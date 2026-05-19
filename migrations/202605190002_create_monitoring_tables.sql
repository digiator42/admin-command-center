
-- Inventory Table
CREATE TABLE IF NOT EXISTS monitored_services (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    target_url TEXT NOT NULL,
    expected_status SMALLINT NOT NULL DEFAULT 200,
    ping_interval_seconds INT NOT NULL DEFAULT 30,
    current_status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Rolling History Log Table
CREATE TABLE IF NOT EXISTS service_heartbeats (
    id SERIAL PRIMARY KEY,
    service_id INT NOT NULL REFERENCES monitored_services(id) ON DELETE CASCADE,
    latency_ms INT NOT NULL,
    http_status_code SMALLINT NOT NULL,
    status_msg VARCHAR(50) NOT NULL,
    error_log TEXT,
    checked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Seed your live Vulntrack Spring Boot backend target into the table!
INSERT INTO monitored_services (name, target_url) 
VALUES ('Vulntrack Backend API', 'http://localhost:8081/actuator/health')
ON CONFLICT DO NOTHING;