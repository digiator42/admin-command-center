use bcrypt::{hash, DEFAULT_COST, BcryptError};

pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

/// Generates a cryptographically secure, valid Bcrypt hash from a raw string.
/// Uses a work factor cost of 12 for robust protection against brute-force attacks.
pub fn generate_hash(plain_text: &str) -> Result<String, BcryptError> {
    // A cost of 12 takes roughly 100-300ms to calculate depending on hardware,
    // which effectively neutralizes offline hardware acceleration (like GPUs).
    let cost = 12; 
    
    hash(plain_text, cost)
}