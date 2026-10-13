-- monitored service inventory
CREATE TABLE IF NOT EXISTS monitored_services (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    target_url TEXT NOT NULL,
    expected_status SMALLINT NOT NULL DEFAULT 200,
    ping_interval_seconds INT NOT NULL DEFAULT 30,
    current_status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
