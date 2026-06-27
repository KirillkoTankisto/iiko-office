use serde::Deserialize;

use crate::api::api_request::{ApiArgs, ApiRequest};

pub struct GetVersion {
    address: String,
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

impl GetVersion {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
        }
    }

    pub fn run(&self) -> Result<VersionInfo, String> {
        let args = ApiArgs::new([("encoding", "UTF-8")]);
        ApiRequest::new(&self.address, "/resto/get_server_info.jsp", args).run_xml()
    }
}
