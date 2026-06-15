use crate::api::api_request::*;

pub struct Logout {
    address: String,
    token: String,
}

impl Logout {
    pub fn new<S: Into<String>>(address: S, token: S) -> Self {
        Self {
            address: address.into(),
            token: token.into(),
        }
    }

    pub fn run(&self) -> Result<(), String> {
        let args = ApiArgs::new([("key", &self.token)]);
        _ = ApiRequest::new(self.address.clone(), "/resto/api/logout", args).run_string()?;
        Ok(())
    }
}
