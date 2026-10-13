-- legacy vulntrack placeholder probe
INSERT INTO monitored_services (name, target_url)
SELECT 'Vulntrack Backend API', 'http://localhost:8081/actuator/health'
WHERE NOT EXISTS (
    SELECT 1 FROM monitored_services
    WHERE target_url = 'http://localhost:8081/actuator/health'
);
