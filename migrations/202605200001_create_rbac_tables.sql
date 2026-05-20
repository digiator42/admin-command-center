
-- Create a secure enum or text validation constraint for our fixed RBAC roles
CREATE TABLE IF NOT EXISTS system_users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'Auditor', -- 'SuperAdmin', 'Operator', 'Auditor'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_role CHECK (role IN ('SuperAdmin', 'Operator', 'Auditor'))
);

-- State-backed Session table for secure cookie token verification
CREATE TABLE IF NOT EXISTS user_sessions (
    id VARCHAR(255) PRIMARY KEY, -- Secure random session token UUID / string
    user_id INT NOT NULL REFERENCES system_users(id) ON DELETE CASCADE,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Seed a default SuperAdmin account for testing setup (Password: admin123)
INSERT INTO system_users (username, password_hash, role)
VALUES ('admin', '$2b$12$K7v1bO8qWv3P5X2yKzZ1eOaL7M2mGvHn8JqK5rL4M3N2O1P0Q.RS.', 'SuperAdmin')
ON CONFLICT DO NOTHING;