use sha1::{Digest, Sha1};

use crate::api::{api_request::*, error::ClientError};

pub struct Auth<'a> {
    address: &'a str,
    user: &'a str,
    pass: &'a str,
}

impl<'a> Auth<'a> {
    pub fn new(address: &'a str, user: &'a str, pass: &'a str) -> Self {
        Self {
            address,
            user,
            pass,
        }
    }

    pub fn run(&self) -> Result<String, ClientError> {
        let args = ApiArgs::new([("login", self.user), ("pass", self.pass)]);
        ApiRequest::new(self.address, "/resto/api/auth", args).run_string()
    }
}

pub fn get_password_hash(password: &str) -> String {
    let password_hashed: String = Sha1::digest(password.as_bytes())
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect();

    password_hashed
}
