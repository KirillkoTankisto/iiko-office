use std::sync::Mutex;

use crate::{IikoConnection, IikoSession, error::ClientError};

impl IikoConnection {
    pub fn auth(self, login: &str, password: &str) -> Result<IikoSession, ClientError> {
        let token =
            self.request_string("/resto/api/auth", &[("login", login), ("pass", password)])?;

        Ok(IikoSession {
            connection: self,
            user: login.to_string(),
            hashed_password: password.to_string(),
            token: Mutex::new(token),
        })
    }
}
