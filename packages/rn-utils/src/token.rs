use rand::Rng;
use sha2::{Digest, Sha256};

/// Generates a cryptographically secure random hex string of the specified byte length.
pub fn generate_secure_token(bytes: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut buffer = vec![0u8; bytes];
    rng.fill(&mut buffer[..]);
    hex::encode(buffer)
}

/// Generates a random numeric OTP string of the specified digit length.
pub fn generate_numeric_otp(digits: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..digits)
        .map(|_| rng.gen_range(0..=9).to_string())
        .collect()
}

/// Computes the SHA-256 hex hash of an input string.
pub fn sha256_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}
