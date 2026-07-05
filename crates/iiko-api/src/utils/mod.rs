use sha1::{Digest, Sha1};

pub fn get_password_hash(password: &str) -> String {
    Sha1::digest(password.as_bytes())
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}
