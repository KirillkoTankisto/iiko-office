use sha1::{Digest, Sha1};

use crate::api::api_request::*;

pub struct Auth {
    pub address: String,
    pub user: String,
    pub pass: String,
}

impl Auth {
    pub fn new<S: Into<String>>(address: S, user: S, pass: S) -> Self {
        Self {
            address: address.into(),
            user: user.into(),
            pass: pass.into(),
        }
    }

    pub fn run(&self) -> Result<String, String> {
        let args = ApiArgs::new([("login", &self.user), ("pass", &self.pass)]);
        ApiRequest::new(self.address.clone(), "/resto/api/auth", args).run_string()
    }
}

pub fn get_password_hash(password: &str) -> String {
    let password_hashed: String = Sha1::digest(password.as_bytes())
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect();

    password_hashed
}
