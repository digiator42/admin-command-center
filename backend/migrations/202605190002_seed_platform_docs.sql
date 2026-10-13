-- default onboarding document
INSERT INTO platform_docs (slug, title, markdown_body) VALUES (
    'security/rate-limiting',
    'GritShield Perimeter Hardening Guide',
    'Rate limiting inside GritShield is managed via thread-safe atomic window counters. Configure your bucket tokens directly in the middleware initialization loop.'
) ON CONFLICT DO NOTHING;
