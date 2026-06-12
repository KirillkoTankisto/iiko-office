use std::collections::HashMap;

use serde::{Deserialize};
use strum_macros::Display;

use crate::api::api_request::{ApiArgs, ApiRequest};

#[derive(Clone, Copy, Display)]
pub enum ReportType {
    SALES,
    TRANSACTIONS,
    DELIVERIES,
}

pub struct OlapColumnsRequest {
    address: String,
    token: String,
    report_type: ReportType,
}

#[allow(nonstandard_style)]
#[derive(Clone, Copy, Display, Deserialize)]
pub enum FieldType {
    ENUM,
    STRING,
    ID,
    DATETIME,
    INTEGER,
    PERCENT,
    DURATION_IN_SECONDS,
    AMOUNT,
    MONEY,
}

#[derive(Deserialize)]
pub struct OlapField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: FieldType
}

pub type OlapFields = HashMap<String, OlapField>;

impl OlapColumnsRequest {
    pub fn new(address: impl Into<String>, token: impl Into<String>, report_type: ReportType) -> Self
    {
        Self {
            address: address.into(),
            token: token.into(),
            report_type
        }
    }

    pub fn run(&self) -> Result<HashMap<String, OlapField>, String> {
        let args = ApiArgs::new([("key", self.token.clone()), ("reportType", self.report_type.to_string())]);

        ApiRequest::new(self.address.clone(), "/resto/api/v2/olap/columns", args).run::<OlapFields>()
    }
}
