use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::api::{
    api_request::{ApiArgs, ApiRequest},
    consts::{EnumRange, FilterType, PeriodType},
    error::ClientError,
    olap::{FilterType::DateRange, PeriodType::CUSTOM},
};

use super::olap_columns::ReportType;

#[allow(nonstandard_style)]
#[derive(Serialize)]
pub struct OlapRequest<'a> {
    #[serde(skip_serializing)]
    address: &'a str,
    #[serde(skip_serializing)]
    token: &'a str,
    reportType: ReportType,
    #[serde(skip_serializing_if = "Option::is_none")]
    buildSummary: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    groupByRowFields: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    groupByColFields: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    aggregateFields: Vec<String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    filters: IndexMap<String, Filter>,
}

#[allow(nonstandard_style)]
#[derive(Serialize)]
pub struct Filter {
    filterType: FilterType,
    #[serde(skip_serializing_if = "Option::is_none")]
    periodType: Option<PeriodType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<EnumRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<EnumRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    includeLow: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    includeHigh: Option<bool>,
}

#[allow(nonstandard_style)]
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
        (
            field_name.into(),
            Self {
                filterType,
                periodType,
                values,
                from,
                to,
                includeLow,
                includeHigh,
            },
        )
    }

    pub fn new_date_range(from: String, to: String) -> (String, Filter) {
        (
            String::from("OpenDate.Typed"),
            Self {
                filterType: DateRange,
                periodType: Some(CUSTOM),
                values: None,
                from: Some(EnumRange::String(from)),
                to: Some(EnumRange::String(to)),
                includeLow: None,
                includeHigh: None,
            },
        )
    }

    pub fn preset_date_range(period_type: PeriodType) -> (String, Filter) {
        (
            String::from("OpenDate.Typed"),
            Self {
                filterType: DateRange,
                periodType: Some(period_type),
                values: None,
                from: None,
                to: None,
                includeLow: None,
                includeHigh: None,
            },
        )
    }
}

pub type SummaryBlock = Vec<IndexMap<String, String>>;

#[derive(Deserialize, Debug)]
pub struct OlapAnswer {
    pub data: Vec<IndexMap<String, serde_json::Value>>,
    pub summary: Vec<SummaryBlock>,
}

impl<'a> OlapRequest<'a> {
    pub fn new(
        address: &'a str,
        token: &'a str,
        report_type: ReportType,
        build_summary: Option<bool>,
        group_by_row_fields: Vec<String>,
        group_by_col_fields: Vec<String>,
        aggregate_fields: Vec<String>,
        filters: IndexMap<String, Filter>,
    ) -> Self {
        Self {
            address,
            token,
            reportType: report_type,
            buildSummary: build_summary,
            groupByRowFields: group_by_row_fields,
            groupByColFields: group_by_col_fields,
            aggregateFields: aggregate_fields,
            filters,
        }
    }

    pub fn run(&self) -> Result<OlapAnswer, ClientError> {
        let args = ApiArgs::new([("key", self.token)]);

        let body = serde_json::to_string_pretty(&self)?;

        ApiRequest::new(self.address, "/resto/api/v2/reports/olap", args).run_post(body)
    }
}
