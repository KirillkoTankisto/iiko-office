use indexmap::IndexMap;
use serde::Deserialize;

use crate::{IikoSession, consts::ReportType, error::ClientError};

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OlapColumn {
    pub name: String,
    #[serde(rename = "type")]
    pub column_type: String,
    pub aggregation_allowed: bool,
    pub grouping_allowed: bool,
    pub filtering_allowed: bool,
    pub tags: Vec<String>,
}

pub type OlapColumns = IndexMap<String, OlapColumn>;

impl IikoSession {
    pub fn olap_columns(&self, report_type: ReportType) -> Result<OlapColumns, ClientError> {
        self.request_json(
            "/resto/api/v2/reports/olap/columns",
            &[("reportType", report_type.as_str())],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IikoConnection;
    use httpmock::prelude::*;
    use std::sync::Mutex;

    const OLAP_COLUMNS: &str = include_str!("../../tests/olap_columns.json");
    const KEY: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
    const PASSWORD: &str = "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8";
    const USER: &str = "admin";
    const REPORT_TYPE: &str = "SALES";

    #[test]
    fn olap_columns_get() {
        let server = httpmock::MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/resto/api/v2/reports/olap/columns")
                .query_param("key", KEY)
                .query_param("reportType", REPORT_TYPE);
            then.status(200).body(OLAP_COLUMNS);
        });

        let session = IikoSession {
            connection: IikoConnection::new(&server.base_url()).unwrap(),
            user: USER.to_string(),
            hashed_password: PASSWORD.to_string(),
            token: Mutex::new(KEY.to_string()),
        };

        let answer = session.olap_columns(ReportType::Sales).unwrap();

        mock.assert();

        assert_eq!(
            answer,
            serde_json::from_str::<OlapColumns>(OLAP_COLUMNS).unwrap()
        );
    }
}
