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
    const DATE_STUB: &str = "2000-01-01T00:00:00.000";

    pub fn custom_date_range(from: String, to: String) -> Self {
        Self::DateRange {
            period_type: PeriodType::Custom,
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

#[derive(Clone, Copy, Default)]
pub enum OlapRowKind {
    #[default]
    Data,
    Subtotal {
        level: usize,
    },
    GrandTotal,
}

pub struct OlapTable {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_kinds: Vec<OlapRowKind>,
    pub key_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct GroupOptions<'a> {
    pub total_label: &'a str,
    pub blank_repeats: bool,
    pub subtotals: bool,
    pub grand_total: bool,
}

impl<'a> GroupOptions<'a> {
    pub fn grouped(total_label: &'a str) -> Self {
        Self {
            total_label,
            blank_repeats: true,
            subtotals: true,
            grand_total: false,
        }
    }

    pub fn with_grand_total(mut self, grand_total: bool) -> Self {
        self.grand_total = grand_total;
        self
    }
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

        // Insert total_label as the first item in a Totals row
        if let Some(first) = totals.first_mut() {
            *first = total_label.to_string();
        }

        totals.extend(column_totals.into_iter().map(Self::fmt_cell));
        rows.push(totals);

        let mut row_kinds = vec![OlapRowKind::Data; rows.len()];
        if let Some(last) = row_kinds.last_mut() {
            *last = OlapRowKind::GrandTotal;
        }

        OlapTable {
            columns,
            rows,
            row_kinds,
            key_count: row_fields.len(),
        }
    }

    pub fn to_table_grouped(&self, row_fields: &[String], opts: GroupOptions<'_>) -> OlapTable {
        let (all_columns, flat_rows) = self.flatten_records();

        let mut columns: IndexSet<String> = IndexSet::new();
        for field in row_fields {
            let hit = all_columns
                .iter()
                .find(|c| *c == field)
                .or_else(|| all_columns.iter().find(|c| c.starts_with(field)));
            if let Some(name) = hit {
                columns.insert(name.clone());
            }
        }

        let key_count = columns.len();
        columns.extend(all_columns.iter().cloned());

        let columns: Vec<String> = columns.into_iter().collect();
        let width = columns.len();

        let rows: Vec<Vec<String>> = flat_rows
            .into_iter()
            .map(|flat| {
                columns
                    .iter()
                    .map(|c| flat.get(c).cloned().unwrap_or_default())
                    .collect()
            })
            .collect();

        let mut order: Vec<usize> = (0..rows.len()).collect();
        order.sort_by(|&a, &b| Self::cmp_keys(&rows[a][..key_count], &rows[b][..key_count]));

        let mut out = OlapTable {
            columns,
            rows: Vec::with_capacity(order.len()),
            row_kinds: Vec::with_capacity(order.len()),
            key_count,
        };

        let mut group_start = vec![0usize; key_count];

        for pos in 0..order.len() {
            let row = &rows[order[pos]];

            let start_level = if pos == 0 {
                0
            } else {
                let prev = &rows[order[pos - 1]];
                (0..key_count)
                    .find(|&d| row[d] != prev[d])
                    .unwrap_or(key_count)
            };

            if pos > 0 && opts.subtotals && start_level < key_count {
                Self::close_groups(
                    &mut out,
                    &rows,
                    &order,
                    &group_start,
                    start_level,
                    pos,
                    &opts,
                );
            }
            for slot in group_start.iter_mut().skip(start_level) {
                *slot = pos;
            }

            let mut cells = row.clone();
            if opts.blank_repeats {
                for cell in cells.iter_mut().take(start_level) {
                    cell.clear();
                }
            }
            out.rows.push(cells);
            out.row_kinds.push(OlapRowKind::Data);
        }

        if !order.is_empty() {
            if opts.subtotals {
                Self::close_groups(&mut out, &rows, &order, &group_start, 0, order.len(), &opts);
            }
            if opts.grand_total {
                let sums: Vec<Option<f64>> = (key_count..width)
                    .map(|col| Self::sum_column(&rows, &order, col))
                    .collect();

                if sums.iter().any(Option::is_some) {
                    let mut row = vec![String::new(); width];
                    row[0] = opts.total_label.to_string();
                    for (col, sum) in (key_count..width).zip(sums) {
                        if let Some(sum) = sum {
                            row[col] = Self::fmt_cell(sum);
                        }
                    }
                    out.rows.push(row);
                    out.row_kinds.push(OlapRowKind::GrandTotal);
                }
            }
        }

        out
    }

    fn close_groups(
        out: &mut OlapTable,
        rows: &[Vec<String>],
        order: &[usize],
        group_start: &[usize],
        from_level: usize,
        end_pos: usize,
        opts: &GroupOptions<'_>,
    ) {
        let width = out.columns.len();
        let key_count = out.key_count;
        // The innermost level gets no subtotal: its groups are single rows.
        for level in (from_level..key_count.saturating_sub(1)).rev() {
            let start = group_start[level];
            if end_pos.saturating_sub(start) < 2 {
                continue; // No total is needed
            }

            let sums: Vec<Option<f64>> = (key_count..width)
                .map(|col| Self::sum_column(rows, &order[start..end_pos], col))
                .collect();

            if sums.iter().all(Option::is_none) {
                continue;
            }

            let mut row = vec![String::new(); width];
            row[level] = format!("{} {}", rows[order[start]][level], opts.total_label);
            for (col, sum) in (key_count..width).zip(sums) {
                if let Some(sum) = sum {
                    row[col] = Self::fmt_cell(sum);
                }
            }
            out.rows.push(row);
            out.row_kinds.push(OlapRowKind::Subtotal { level });
        }
    }

    fn sum_column(rows: &[Vec<String>], members: &[usize], col: usize) -> Option<f64> {
        let mut acc = None;
        for &i in members {
            let cell = rows[i]
                .get(col)
                .map(String::as_str)
                .unwrap_or_default()
                .trim();
            if cell.is_empty() {
                continue;
            }
            acc = Some(acc.unwrap_or(0.0) + Self::parse_num(cell)?);
        }
        acc
    }

    fn flatten_records(&self) -> (Vec<String>, Vec<IndexMap<String, String>>) {
        let mut column_set: IndexSet<String> = IndexSet::new();
        let mut flat_rows: Vec<IndexMap<String, String>> = Vec::with_capacity(self.data.len());

        for record in &self.data {
            let mut flat = IndexMap::new();
            for (key, value) in record {
                Self::flatten(key, value, &mut flat);
            }
            column_set.extend(flat.keys().cloned());
            flat_rows.push(flat);
        }

        (column_set.into_iter().collect(), flat_rows)
    }

    /* Sorting functions */

    fn cmp_key(a: &str, b: &str) -> Ordering {
        let (ka, kb) = (Self::classify(a), Self::classify(b));
        match (ka, kb) {
            (CellKey::Empty, CellKey::Empty) => Ordering::Equal,
            (CellKey::Number(x), CellKey::Number(y)) => x.total_cmp(&y),
            (CellKey::Date(x), CellKey::Date(y)) => x.cmp(&y),
            (CellKey::Text, CellKey::Text) => Self::cmp_ci(a, b),
            _ => ka.rank().cmp(&kb.rank()),
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

    // case-insensitive compare, without allocating
    fn cmp_ci(a: &str, b: &str) -> Ordering {
        a.chars()
            .flat_map(char::to_lowercase)
            .cmp(b.chars().flat_map(char::to_lowercase))
    }

    // Classify the entry
    fn classify(s: &str) -> CellKey {
        let t = s.trim();
        if t.is_empty() {
            CellKey::Empty
        } else if let Some(d) = Self::parse_date(t) {
            CellKey::Date(d)
        } else if let Some(n) = Self::parse_num(t) {
            CellKey::Number(n)
        } else {
            CellKey::Text
        }
    }

    // parses date as an array. If the input is not an array, returns None
    fn parse_date(s: &str) -> Option<[u16; 6]> {
        let s = s.trim_end_matches('Z');
        let (date, time) = match s.split_once(['T', ' ']) {
            Some((d, t)) => (d, Some(t)),
            None => (s, None),
        };

        let (y, m, d) = if let Some((a, rest)) = date.split_once('-') {
            let (b, c) = rest.split_once('-')?;
            (Self::field(a, 4)?, Self::field(b, 2)?, Self::field(c, 2)?)
        } else if let Some((a, rest)) = date.split_once('.') {
            let (b, c) = rest.split_once('.')?;
            (Self::field(c, 4)?, Self::field(b, 2)?, Self::field(a, 2)?)
        } else {
            return None;
        };
        if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
            return None;
        }

        let mut out = [y, m, d, 0, 0, 0];
        if let Some(t) = time {
            let t = t.split('.').next().unwrap_or(t); // drop fractional seconds
            for (i, part) in t.split(':').enumerate() {
                *out.get_mut(3 + i)? = Self::field(part, 2)?;
            }
        }
        Some(out)
    }

    // Parse date field as a number
    fn field(s: &str, max_len: usize) -> Option<u16> {
        if s.is_empty() || s.len() > max_len || !s.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        s.parse().ok()
    }

    fn parse_num(t: &str) -> Option<f64> {
        let first = *t.as_bytes().first()?;
        if !(first.is_ascii_digit() || first == b'-' || first == b'+') {
            return None;
        }
        let n: f64 = t.replace(',', ".").parse().ok()?;
        n.is_finite().then_some(n)
    }

    /* Helpers */

    // Json value to an owned string
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

    // Stringify a json value (String and Null, specifically)
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

    // f64 to String. Empty if zero
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum CellKey {
    Empty,
    Number(f64),
    /// year, month, day, hour, minute, second
    Date([u16; 6]),
    Text,
}

impl CellKey {
    /// Different types have different weight.
    /// Rank returns the weight of the value
    fn rank(self) -> u8 {
        match self {
            CellKey::Empty => 0,
            CellKey::Number(_) => 1,
            CellKey::Date(_) => 2,
            CellKey::Text => 3,
        }
    }
}
