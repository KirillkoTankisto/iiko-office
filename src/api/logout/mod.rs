use std::error::Error;

use crate::api::api_request::*;

pub struct Logout {
    pub address: String,
    pub token: String,
}

impl Logout {
    pub fn new<S: Into<String>>(address: S, token: S) -> Self {
        Self {
            address: address.into(),
            token: token.into(),
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let args = ApiArgs::new([("key", &self.token)]);
        _ = ApiRequest::new(self.address.clone(), "/resto/api/logout", args).run_string()?;
        Ok(())
    }
}
