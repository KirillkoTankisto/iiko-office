use std::error::Error;

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

    pub fn run(&self) -> Result<String, Box<dyn Error>> {
        let args = ApiArgs::new([("user", &self.user), ("pass", &self.pass)]);
        Ok(ApiRequest::new(self.address.clone(), "/resto/api/auth".into(), args).run_string()?)
    }
}

pub fn get_password_hash(password: &String) -> String {
    let password_hashed: String = Sha1::digest(password.as_bytes())
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect();

    return password_hashed;
}
