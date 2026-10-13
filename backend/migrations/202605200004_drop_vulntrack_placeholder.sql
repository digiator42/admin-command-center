-- retire legacy vulntrack probe
DELETE FROM monitored_services
WHERE target_url = 'http://localhost:8081/actuator/health';
