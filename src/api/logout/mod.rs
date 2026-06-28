use crate::api::{api_request::*, error::ClientError};

pub struct Logout<'a> {
    address: &'a str,
    token: &'a str,
}

impl<'a> Logout<'a> {
    pub fn new(address: &'a str, token: &'a str) -> Self {
        Self { address, token }
    }

    pub fn run(&self) -> Result<(), ClientError> {
        let args = ApiArgs::new([("key", self.token)]);
        _ = ApiRequest::new(self.address, "/resto/api/logout", args).run_string()?;
        Ok(())
    }
}
