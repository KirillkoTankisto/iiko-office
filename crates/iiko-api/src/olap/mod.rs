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

pub struct OlapTable {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl OlapAnswer {
    pub fn to_table(&self) -> OlapTable {
        let mut columns: IndexMap<String, ()> = IndexMap::new();
        let mut flat_rows: Vec<IndexMap<String, String>> = Vec::with_capacity(self.data.len());
        for record in &self.data {
            let mut flat = IndexMap::new();
            for (key, value) in record {
                Self::flatten(key, value, &mut flat);
            }
            for col in flat.keys() {
                columns.entry(col.clone()).or_insert(());
            }
            flat_rows.push(flat);
        }

        let column_list: Vec<String> = columns.into_keys().collect();
        let rows = flat_rows
            .into_iter()
            .map(|flat| {
                column_list
                    .iter()
                    .map(|c| flat.get(c).cloned().unwrap_or_default())
                    .collect()
            })
            .collect();

        OlapTable {
            columns: column_list,
            rows,
        }
    }

    fn flatten(prefix: &str, value: &serde_json::Value, out: &mut IndexMap<String, String>) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    Self::flatten(&format!("{prefix} / {key}"), value, out);
                }
            }
            serde_json::Value::Array(arr) => {
                let joined = arr
                    .iter()
                    .map(Self::value_to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                out.insert(prefix.to_string(), joined);
            }
            scalar => {
                out.insert(prefix.to_string(), Self::value_to_string(scalar));
            }
        }
    }

    fn value_to_string(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null => String::new(),
            other => other.to_string(),
        }
    }
}
