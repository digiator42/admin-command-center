-- reconcile seeded credential hashes
INSERT INTO system_users (username, password_hash, role) VALUES
    ('admin',    '$2b$12$avlbYtsCVZizd5/isBOuieVaWIy9CTQr5/cd0Ms2w.Q79OolkQbMq', 'Admin'),
    ('operator', '$2b$12$i0BTQO9EEDetROHW8VZYCOGyxVxCXPanKPh3lo1gym/N510CN4uYi', 'Operator'),
    ('auditor',  '$2b$12$1YJZEVGoa25SKW6NXvDAi.FTM0vRe6hxLSc68RHnyVBWr4j7IZUm.', 'Auditor')
ON CONFLICT (username) DO UPDATE
    SET password_hash = EXCLUDED.password_hash,
        role = EXCLUDED.role;
