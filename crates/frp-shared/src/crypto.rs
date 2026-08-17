use sha2::{Digest, Sha256};

/// Hash a sensitive token using SHA256 for safe storage or comparison.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Compare two token hashes in constant time to prevent timing attacks.
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    if a_bytes.len() != b_bytes.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a_bytes.iter().zip(b_bytes.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// Verify if a provided raw token matches the stored SHA256 hash.
pub fn verify_token(raw_token: &str, stored_hash: &str) -> bool {
    let raw_hash = hash_token(raw_token);
    constant_time_eq(&raw_hash, stored_hash)
}

/// Mask sensitive values for structured logging (showing only the last 4 characters).
pub fn mask_secret(secret: &str) -> String {
    if secret.len() <= 4 {
        "****".to_string()
    } else {
        format!("****{}", &secret[secret.len() - 4..])
    }
}
