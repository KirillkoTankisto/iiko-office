use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::api::{
    api_request::{ApiArgs, ApiRequest},
    error::ClientError,
};

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

pub struct OlapColumnRequest<'a> {
    address: &'a str,
    token: &'a str,
    report_type: String,
}

impl<'a> OlapColumnRequest<'a> {
    pub fn new(address: &'a str, token: &'a str, report_type: ReportType) -> Self {
        Self {
            address,
            token,
            report_type: report_type.to_string(),
        }
    }

    pub fn run(&self) -> Result<OlapColumns, ClientError> {
        let args = ApiArgs::new([
            ("key", self.token),
            ("reportType", self.report_type.as_str()),
        ]);
        ApiRequest::new(self.address, "/resto/api/v2/reports/olap/columns", args)
            .run::<OlapColumns>()
    }
}
