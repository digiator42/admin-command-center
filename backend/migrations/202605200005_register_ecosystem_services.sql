-- register product health probes
INSERT INTO monitored_services (name, target_url)
SELECT v.name, v.target_url
FROM (VALUES
    ('CyberFlow Backend API', 'http://cyberflow-backend:8010/health'),
    ('LuxShield Backend API', 'http://luxshield-backend:8020/health'),
    ('GritIRT Backend API',   'http://gritirt-backend:8030/health'),
    ('GritJira Backend API',  'http://gritjira-backend:8040/health')
) AS v(name, target_url)
WHERE NOT EXISTS (
    SELECT 1 FROM monitored_services
    WHERE target_url = v.target_url
);
