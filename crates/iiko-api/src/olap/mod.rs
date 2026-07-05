use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    IikoSession,
    consts::{
        EnumRange,
        FilterType::{self, DateRange},
        PeriodType, ReportType,
    },
    error::ClientError,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OlapRequest {
    report_type: ReportType,
    build_summary: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    group_by_row_fields: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    group_by_col_fields: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    aggregate_fields: Vec<String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    filters: IndexMap<String, Filter>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    filter_type: FilterType,
    #[serde(skip_serializing_if = "Option::is_none")]
    period_type: Option<PeriodType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<EnumRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<EnumRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_low: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_high: Option<bool>,
}

#[allow(nonstandard_style)]
impl Filter {
    pub fn new_date_range(from: String, to: String) -> (String, Filter) {
        (
            String::from("OpenDate.Typed"),
            Self {
                filter_type: DateRange,
                period_type: Some(PeriodType::CUSTOM),
                values: None,
                from: Some(EnumRange::String(from)),
                to: Some(EnumRange::String(to)),
                include_low: None,
                include_high: None,
            },
        )
    }

    pub fn preset_date_range(period_type: PeriodType) -> (String, Filter) {
        (
            String::from("OpenDate.Typed"),
            Self {
                filter_type: DateRange,
                period_type: Some(period_type),
                values: None,
                from: Some(EnumRange::String(String::from("2000-01-01T00:00:00.000"))), // an iikoServer API quirk
                to: None,
                include_low: None,
                include_high: None,
            },
        )
    }
}

pub type SummaryBlock = Vec<IndexMap<String, String>>;

#[derive(Deserialize)]
pub struct OlapAnswer {
    pub data: Vec<IndexMap<String, serde_json::Value>>,
    pub summary: Vec<SummaryBlock>,
}

impl IikoSession {
    pub fn olap(
        &self,
        report_type: ReportType,
        build_summary: bool,
        group_by_row_fields: Vec<String>,
        group_by_col_fields: Vec<String>,
        aggregate_fields: Vec<String>,
        filters: IndexMap<String, Filter>,
    ) -> Result<OlapAnswer, ClientError> {
        let body = serde_json::to_string_pretty(&OlapRequest {
            report_type,
            build_summary,
            group_by_row_fields,
            group_by_col_fields,
            aggregate_fields,
            filters,
        })?;

        self.request_post("/resto/api/v2/reports/olap", &[], body)
    }
}
