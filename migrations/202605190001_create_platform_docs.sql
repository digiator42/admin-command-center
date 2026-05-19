CREATE TABLE IF NOT EXISTS platform_docs (
    id SERIAL PRIMARY KEY,
    slug VARCHAR(255) UNIQUE NOT NULL, -- e.g., 'security/rate-limiting'
    title VARCHAR(255) NOT NULL,
    markdown_body TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert a premium test document to verify the fallback live!
INSERT INTO platform_docs (slug, title, markdown_body) VALUES (
    'security/rate-limiting',
    'GritShield Perimeter Hardening Guide',
    'Rate limiting inside GritShield is managed via thread-safe atomic window counters. Configure your bucket tokens directly inside the middleware initialization loop.'
) ON CONFLICT DO NOTHING;