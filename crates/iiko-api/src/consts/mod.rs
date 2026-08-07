use serde::Serialize;

use crate::macros::str_enum;

str_enum! {
    pub enum PeriodType {
        Custom => "CUSTOM",
        OpenPeriod => "OPEN_PERIOD",
        Today => "TODAY",
        Yesterday => "YESTERDAY",
        CurrentWeek => "CURRENT_WEEK",
        CurrentMonth => "CURRENT_MONTH",
        CurrentYear => "CURRENT_YEAR",
        LastWeek => "LAST_WEEK",
        LastMonth => "LAST_MONTH",
        LastYear => "LAST_YEAR",
    }
}

str_enum! {
    pub enum ReportType {
        Sales => "SALES",
        Transactions => "TRANSACTIONS",
        Deliveries => "DELIVERIES",
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum EnumRange {
    I64(i64),
    F64(f64),
    String(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_strings_are_stable() {
        assert_eq!(PeriodType::OpenPeriod.as_str(), "OPEN_PERIOD");
        assert_eq!(PeriodType::CurrentWeek.as_str(), "CURRENT_WEEK");
        assert_eq!(ReportType::Sales.as_str(), "SALES");
        assert_eq!(
            serde_json::to_string(&PeriodType::LastYear).unwrap(),
            "\"LAST_YEAR\""
        );
    }
}
