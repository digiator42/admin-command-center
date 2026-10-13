-- bootstrap administrator account
INSERT INTO system_users (username, password_hash, role)
VALUES ('admin', '$2b$12$K7v1bO8qWv3P5X2yKzZ1eOaL7M2mGvHn8JqK5rL4M3N2O1P0Q.RS.', 'Admin')
ON CONFLICT DO NOTHING;
