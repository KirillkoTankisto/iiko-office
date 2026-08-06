use serde::Serialize;

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PeriodType {
    Custom,
    OpenPeriod,
    Today,
    Yesterday,
    CurrentWeek,
    CurrentMonth,
    CurrentYear,
    LastWeek,
    LastMonth,
    LastYear,
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
