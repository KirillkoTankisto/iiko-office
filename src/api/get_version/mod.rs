use serde::Deserialize;

use crate::api::{
    api_request::{ApiArgs, ApiRequest},
    error::ClientError,
};

pub struct GetVersion<'a> {
    address: &'a str,
}

#[allow(nonstandard_style)]
#[derive(Deserialize)]
pub struct VersionInfo {
    pub serverName: String,
    pub edition: String,
    pub version: String,
    pub computerName: String,
    pub serverState: String,
}

impl<'a> GetVersion<'a> {
    pub fn new(address: &'a str) -> Self {
        Self { address }
    }

    pub fn run(&self) -> Result<VersionInfo, ClientError> {
        let args = ApiArgs::new([("encoding", "UTF-8")]);
        ApiRequest::new(self.address, "/resto/get_server_info.jsp", args).run_xml()
    }
}
