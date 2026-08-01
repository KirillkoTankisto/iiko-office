use std::cmp::Ordering;

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    IikoSession,
    consts::{EnumRange, PeriodType, ReportType},
    error::ClientError,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OlapRequest {
    pub report_type: ReportType,
    pub build_summary: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub group_by_row_fields: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub group_by_col_fields: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub aggregate_fields: Vec<String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub filters: IndexMap<String, Filter>,
}

#[derive(Serialize)]
#[serde(tag = "filterType", rename_all_fields = "camelCase")]
pub enum Filter {
    IncludeValues {
        values: Vec<String>,
    },
    ExcludeValues {
        values: Vec<String>,
    },
    DateRange {
        period_type: PeriodType,
        from: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        to: Option<String>,
    },
    ValueRange {
        from: EnumRange,
        to: EnumRange,
        include_low: bool,
        include_high: bool,
    },
}

impl Filter {
    pub const OPEN_DATE_FIELD: &str = "OpenDate.Typed";
    pub const DATE_STUB: &str = "2000-01-01T00:00:00.000";

    pub fn custom_date_range(from: String, to: String) -> Self {
        Self::DateRange {
            period_type: PeriodType::CUSTOM,
            from,
            to: Some(to),
        }
    }

    pub fn preset_date_range(period_type: PeriodType) -> Self {
        Self::DateRange {
            period_type,
            from: Self::DATE_STUB.to_string(),
            to: None,
        }
    }

    pub fn closed_value_range(from: EnumRange, to: EnumRange) -> Self {
        Self::ValueRange {
            from,
            to,
            include_low: true,
            include_high: true,
        }
    }
}

pub type SummaryBlock = Vec<IndexMap<String, String>>;

#[derive(Deserialize, Debug)]
pub struct OlapAnswer {
    pub data: Vec<IndexMap<String, Value>>,
    pub summary: Vec<SummaryBlock>,
}

impl IikoSession {
    pub fn olap(&self, request: &OlapRequest) -> Result<OlapAnswer, ClientError> {
        let body = serde_json::to_string(request)?;
        self.request_post("/resto/api/v2/reports/olap", &[], body)
    }
}

pub struct OlapTable {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl OlapAnswer {
    pub fn to_pivot_table(
        &self,
        row_fields: &[String],
        col_field: &str,
        value_field: &str,
        total_label: &str,
    ) -> OlapTable {
        let mut column_keys: IndexSet<String> = IndexSet::new();
        // row key -> column name -> aggregate
        let mut cells: IndexMap<Vec<String>, IndexMap<String, f64>> = IndexMap::new();

        for record in &self.data {
            let Some(value) = record.get(col_field) else {
                continue;
            };

            let keys: Vec<String> = row_fields
                .iter()
                .map(|f| record.get(f).map(Self::cell_string).unwrap_or_default())
                .collect();
            let row_sums = cells.entry(keys).or_default();

            let mut add = |col: String, val: f64| {
                column_keys.insert(col.clone());
                *row_sums.entry(col).or_insert(0.0) += val;
            };

            match value {
                // Dig one level deeper
                Value::Object(map) => {
                    for (col, sub) in map {
                        add(
                            col.clone(),
                            Self::cell_f64(sub.get(value_field).unwrap_or(sub)),
                        );
                    }
                }
                // Flat value
                other => add(
                    Self::cell_string(other),
                    record.get(value_field).map_or(0.0, Self::cell_f64),
                ),
            }
        }

        let mut column_headers: Vec<String> = column_keys.into_iter().collect();
        column_headers.sort_by(|a, b| Self::cmp_key(a, b));
        cells.sort_by(|a, _, b, _| Self::cmp_keys(a, b));

        let mut columns = row_fields.to_vec();
        columns.extend_from_slice(&column_headers);

        let mut column_totals = vec![0.0; column_headers.len()];
        let mut rows: Vec<Vec<String>> = Vec::with_capacity(cells.len() + 1);

        for (keys, row_sums) in cells {
            let mut row = keys;
            for (col, total) in column_headers.iter().zip(&mut column_totals) {
                let v = row_sums.get(col).copied().unwrap_or(0.0);
                *total += v;
                row.push(Self::fmt_cell(v));
            }
            rows.push(row);
        }

        // Totals row
        let mut totals = vec![String::new(); row_fields.len()];

        // Insert total_label as the first item in a row
        if let Some(first) = totals.first_mut() {
            *first = total_label.to_string();
        }

        totals.extend(column_totals.into_iter().map(Self::fmt_cell));
        rows.push(totals);

        OlapTable { columns, rows }
    }

    pub fn to_table_sorted(&self, sort_by: &[String]) -> OlapTable {
        let mut column_set: IndexSet<String> = IndexSet::from([String::new()]);
        let mut flat_rows: Vec<IndexMap<String, String>> = Vec::with_capacity(self.data.len());

        for record in &self.data {
            let mut flat = IndexMap::new();
            for (key, value) in record {
                Self::flatten(key, value, &mut flat);
            }
            column_set.extend(flat.keys().cloned());
            flat_rows.push(flat);
        }

        let columns: Vec<String> = column_set.into_iter().collect();
        let mut rows: Vec<Vec<String>> = flat_rows
            .into_iter()
            .map(|flat| {
                columns
                    .iter()
                    .map(|c| flat.get(c).cloned().unwrap_or_default())
                    .collect()
            })
            .collect();

        let idx: Vec<usize> = sort_by
            .iter()
            .filter_map(|k| {
                let exact = columns.iter().position(|c| c == k);
                exact.or_else(|| columns.iter().position(|c| c.starts_with(k)))
            })
            .collect();

        rows.sort_by(|a, b| {
            idx.iter()
                .map(|&i| Self::cmp_key(&a[i], &b[i]))
                .find(|o| o.is_ne())
                .unwrap_or(Ordering::Equal)
        });

        OlapTable { columns, rows }
    }

    fn value_to_string(value: &Value) -> String {
        match value {
            // Format numbers
            Value::Number(n) => n.as_f64().map_or_else(|| n.to_string(), Self::fmt_num),
            // Join array members
            Value::Array(arr) => arr
                .iter()
                .map(Self::value_to_string)
                .collect::<Vec<_>>()
                .join(", "),
            // Stringify any other value
            other => Self::cell_string(other),
        }
    }

    // Stringify a json value
    fn cell_string(v: &Value) -> String {
        match v {
            Value::String(s) => s.clone(),
            Value::Null => String::new(),
            other => other.to_string(),
        }
    }

    // Json value to f64
    // All unparsable values count as zero
    fn cell_f64(v: &Value) -> f64 {
        match v {
            Value::Number(n) => n.as_f64().unwrap_or(0.0),
            Value::String(s) => s.trim().replace(',', ".").parse().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    fn fmt_cell(v: f64) -> String {
        if v == 0.0 {
            String::new() // Blank
        } else {
            Self::fmt_num(v)
        }
    }

    fn fmt_num(v: f64) -> String {
        if v.fract().abs() < 1e-9 {
            (v.round() as i64).to_string() // round if difference is negligible
        } else {
            format!("{v:.2}")
        }
    }

    // Numeric keys compare numerically, everything else lexicographically
    fn cmp_key(a: &str, b: &str) -> Ordering {
        match (a.parse::<f64>(), b.parse::<f64>()) {
            (Ok(x), Ok(y)) => x.total_cmp(&y),
            _ => a.cmp(b),
        }
    }

    // Same as above but with lists: first difference wins, shorter key first
    fn cmp_keys(a: &[String], b: &[String]) -> Ordering {
        a.iter()
            .zip(b)
            .map(|(x, y)| Self::cmp_key(x, y))
            .find(|o| o.is_ne())
            .unwrap_or_else(|| a.len().cmp(&b.len()))
    }

    fn flatten(prefix: &str, value: &Value, out: &mut IndexMap<String, String>) {
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    Self::flatten(&format!("{prefix} / {key}"), value, out);
                }
            }
            Value::Array(arr) if arr.iter().any(|v| v.is_object() || v.is_array()) => {
                for (i, item) in arr.iter().enumerate() {
                    Self::flatten(&format!("{prefix}[{i}]"), item, out);
                }
            }
            scalar => {
                out.insert(prefix.to_string(), Self::value_to_string(scalar));
            }
        }
    }
}
