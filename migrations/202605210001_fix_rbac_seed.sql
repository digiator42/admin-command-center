-- Repair the RBAC seed data.
--
-- The original RBAC migration inserted a placeholder value in the
-- `password_hash` column for the default admin account; the string was not a
-- valid bcrypt digest, so the account could never authenticate. Replace it with
-- a real cost-12 bcrypt hash for the documented password "admin123" and seed the
-- two lower-privilege demo accounts exercised by the RBAC hierarchy.
--
-- Idempotent: re-running updates the hashes/roles in place.

INSERT INTO system_users (username, password_hash, role) VALUES
    ('admin',    '$2b$12$avlbYtsCVZizd5/isBOuieVaWIy9CTQr5/cd0Ms2w.Q79OolkQbMq', 'Admin'),
    ('operator', '$2b$12$i0BTQO9EEDetROHW8VZYCOGyxVxCXPanKPh3lo1gym/N510CN4uYi', 'Operator'),
    ('auditor',  '$2b$12$1YJZEVGoa25SKW6NXvDAi.FTM0vRe6hxLSc68RHnyVBWr4j7IZUm.', 'Auditor')
ON CONFLICT (username) DO UPDATE
    SET password_hash = EXCLUDED.password_hash,
        role = EXCLUDED.role;
