-- Align the "Monitored Platform Ecosystem" inventory with the live Grit stack.
-- The original monitoring migration seeded a placeholder Vulntrack target that is
-- no longer part of the platform; drop it and register one health probe per product
-- backend. Compose service DNS names are used so the probe resolves from inside the
-- ACC container on the shared GritSoc network.

DELETE FROM monitored_services
WHERE target_url = 'http://localhost:8081/actuator/health';

INSERT INTO monitored_services (name, target_url)
SELECT v.name, v.target_url
FROM (VALUES
    ('CyberFlow Backend API', 'http://cyberflow-backend:8010/health'),
    ('LuxShield Backend API', 'http://luxshield-backend:8020/health'),
    ('GritIRT Backend API',   'http://gritirt-backend:8030/health'),
    ('GritJira Backend API',  'http://gritjira-backend:8040/health')
) AS v(name, target_url)
WHERE NOT EXISTS (
    SELECT 1 FROM monitored_services m WHERE m.target_url = v.target_url
);
