use sha1::{Digest, Sha1};

pub fn get_password_hash(password: &str) -> String {
    Sha1::digest(password.as_bytes())
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::get_password_hash;

    #[test]
    fn sha1_known_values() {
        assert_eq!(
            get_password_hash("password"),
            "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8"
        );
        assert_eq!(
            get_password_hash(""),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
    }

    #[test]
    fn sha1_is_lowercase_hex() {
        get_password_hash("iiko")
            .chars()
            .for_each(|b| assert!(b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
    }

    #[test]
    fn sha1_length() {
        assert_eq!(get_password_hash("iiko").len(), 40);
    }
}
