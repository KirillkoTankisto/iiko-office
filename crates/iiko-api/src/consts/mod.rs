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

impl ReportType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Sales => "SALES",
            Self::Transactions => "TRANSACTIONS",
            Self::Deliveries => "DELIVERIES",
        }
    }
}
