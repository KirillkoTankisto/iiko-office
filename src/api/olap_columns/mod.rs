use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::api::api_request::{ApiArgs, ApiRequest};

#[derive(Serialize, Display)]
pub enum ReportType {
    SALES,
    TRANSACTIONS,
    DELIVERIES,
}

#[derive(Deserialize)]
#[allow(nonstandard_style)]
pub struct OlapColumn {
    pub name: String,
    #[serde(rename = "type")]
    pub columnType: String,
    pub aggregationAllowed: bool,
    pub groupingAllowed: bool,
    pub filteringAllowed: bool,
    pub tags: Vec<String>,
}

pub type OlapColumns = HashMap<String, OlapColumn>;

pub struct OlapColumnRequest {
    address: String,
    token: String,
    report_type: ReportType,
}

impl OlapColumnRequest {
    pub fn new(address: String, token: String, report_type: ReportType) -> Self {
        Self {
            address,
            token,
            report_type,
        }
    }

    pub fn run(&self) -> Result<OlapColumns, String> {
        let args = ApiArgs::new([
            ("key", self.token.clone()),
            ("reportType", self.report_type.to_string()),
        ]);
        ApiRequest::new(
            self.address.clone(),
            "/resto/api/v2/reports/olap/columns",
            args,
        )
        .run::<OlapColumns>()
    }
}
