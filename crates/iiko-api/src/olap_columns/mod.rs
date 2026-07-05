use std::collections::HashMap;

use serde::Deserialize;

use crate::{IikoSession, consts::ReportType, error::ClientError};

#[derive(Deserialize, Clone)]
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

pub type OlapColumns = HashMap<String, OlapColumn>;

impl IikoSession {
    pub fn olap_columns(&self, report_type: ReportType) -> Result<OlapColumns, ClientError> {
        self.request_json(
            "/resto/api/v2/reports/olap/columns",
            &[("reportType", &report_type.to_string())],
        )
    }
}
