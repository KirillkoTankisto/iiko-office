use serde::Deserialize;

use crate::{IikoConnection, IikoSession, error::ClientError};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub server_name: String,
    pub edition: String,
    pub version: String,
    pub computer_name: String,
    pub server_state: String,
}

impl IikoConnection {
    pub fn version(&self) -> Result<VersionInfo, ClientError> {
        self.request_xml("/resto/get_server_info.jsp", &[("encoding", "UTF-8")])
    }
}

impl IikoSession {
    pub fn version(&self) -> Result<VersionInfo, ClientError> {
        self.connection.version()
    }
}
