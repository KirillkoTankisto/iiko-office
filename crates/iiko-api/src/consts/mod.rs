use std::fmt::Display;

use serde::Serialize;

#[allow(nonstandard_style)]
#[derive(Clone, Copy, Serialize)]
pub enum PeriodType {
    CUSTOM,
    OPEN_PERIOD,
    TODAY,
    YESTERDAY,
    CURRENT_WEEK,
    CURRENT_MONTH,
    CURRENT_YEAR,
    LAST_WEEK,
    LAST_MONTH,
    LAST_YEAR,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum EnumRange {
    I64(i64),
    F64(f64),
    String(String),
}

#[derive(Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ReportType {
    Sales,
    Transactions,
    Deliveries,
}

impl Display for ReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportType::Sales => f.write_str("SALES"),
            ReportType::Transactions => f.write_str("TRANSACTIONS"),
            ReportType::Deliveries => f.write_str("DELIVERIES"),
        }
    }
}
