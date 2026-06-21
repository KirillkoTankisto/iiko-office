use indexmap::IndexMap;
use serde::Serialize;

use crate::api::olap::{FilterType::IncludeValues, PeriodType::CUSTOM};

use super::olap_columns::ReportType;

#[allow(nonstandard_style)]
#[derive(Serialize)]
pub struct OlapRequest {
    reportType: ReportType,
    buildSummary: bool,
    groupByRowFields: Vec<String>,
    groupByColFields: Vec<String>,
    aggregateFields: Vec<String>,
    filters: IndexMap<String, Filter>,
}

#[derive(Serialize)]
pub enum FilterType {
    IncludeValues,
    ExcludeValues,
}

#[allow(nonstandard_style)]
#[derive(Serialize)]
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

#[allow(nonstandard_style)]
#[derive(Serialize)]
pub struct Filter {
    filterType: FilterType,
    periodType: Option<PeriodType>,
    values: Option<Vec<String>>,
    from: Option<EnumRange>,
    to: Option<EnumRange>,
    includeLow: Option<bool>,
    includeHigh: Option<bool>,
}

impl Filter {
    pub fn new(
        field_name: impl Into<String>,
        filterType: FilterType,
        periodType: Option<PeriodType>,
        values: Option<Vec<String>>,
        from: Option<EnumRange>,
        to: Option<EnumRange>,
        includeLow: Option<bool>,
        includeHigh: Option<bool>,
    ) -> (String, Filter) {

        (field_name.into(), Self {
            filterType,
            periodType,
            values,
            from,
            to,
            includeLow,
            includeHigh,
        })
    }

    pub fn new_date_range(from: String, to: String) -> (String, Filter)
    {
        (String::from(""), Self {
            filterType: IncludeValues,
            periodType: Some(CUSTOM),
            values: None,
            from: Some(EnumRange::String(from)),
            to: Some(EnumRange::String(to)),
            includeLow: None,
            includeHigh: None,
        })
    }
}

impl OlapRequest {
    pub fn new(report_type: ReportType, build_summary: bool, group_by_row_fields: Vec<String>, group_by_col_fields: Vec<String>, aggregate_fields: Vec<String>, filters: IndexMap<String, Filter>) -> Self {
        todo!("Olap API is not done yet!!!")
    }
}
