mod cell;
mod sort;

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
    #[serde(default)]
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
                .map(|f| record.get(f).map(cell::text).unwrap_or_default())
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
                            cell::number(sub.get(value_field).unwrap_or(sub)),
                        );
                    }
                }
                // Flat value
                other => add(
                    cell::text(other),
                    record.get(value_field).map_or(0.0, cell::number),
                ),
            }
        }

        let mut column_headers: Vec<String> = column_keys.into_iter().collect();
        column_headers.sort_by(|a, b| sort::compare(a, b));
        cells.sort_by(|a, _, b, _| sort::compare_keys(a, b));

        let mut columns = row_fields.to_vec();
        columns.extend_from_slice(&column_headers);

        let mut column_totals = vec![0.0; column_headers.len()];
        let mut rows: Vec<Vec<String>> = Vec::with_capacity(cells.len() + 1);

        for (keys, row_sums) in cells {
            let mut row = keys;
            for (col, total) in column_headers.iter().zip(&mut column_totals) {
                let v = row_sums.get(col).copied().unwrap_or(0.0);
                *total += v;
                row.push(cell::format_total(v));
            }
            rows.push(row);
        }

        // Totals row
        let mut totals = vec![String::new(); row_fields.len()];

        // Insert total_label as the first item in a Totals row
        if let Some(first) = totals.first_mut() {
            *first = total_label.to_string();
        }

        totals.extend(column_totals.into_iter().map(cell::format_total));
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
        order.sort_by(|&a, &b| sort::compare_keys(&rows[a][..key_count], &rows[b][..key_count]));

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
                close_groups(
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
                close_groups(&mut out, &rows, &order, &group_start, 0, order.len(), &opts);
            }
            if opts.grand_total {
                let sums: Vec<Option<f64>> = (key_count..width)
                    .map(|col| sum_column(&rows, &order, col))
                    .collect();

                if sums.iter().any(Option::is_some) {
                    let mut row = vec![String::new(); width];
                    row[0] = opts.total_label.to_string();
                    for (col, sum) in (key_count..width).zip(sums) {
                        if let Some(sum) = sum {
                            row[col] = cell::format_total(sum);
                        }
                    }
                    out.rows.push(row);
                    out.row_kinds.push(OlapRowKind::GrandTotal);
                }
            }
        }

        out
    }

    fn flatten_records(&self) -> (Vec<String>, Vec<IndexMap<String, String>>) {
        let mut column_set: IndexSet<String> = IndexSet::new();
        let mut flat_rows: Vec<IndexMap<String, String>> = Vec::with_capacity(self.data.len());

        for record in &self.data {
            let mut flat = IndexMap::new();
            for (key, value) in record {
                cell::flatten(key, value, &mut flat);
            }
            column_set.extend(flat.keys().cloned());
            flat_rows.push(flat);
        }

        (column_set.into_iter().collect(), flat_rows)
    }
}

/// Emits a subtotal row for every group that ends at 'end_pos'
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

    for level in (from_level..key_count.saturating_sub(1)).rev() {
        let start = group_start[level];
        if end_pos.saturating_sub(start) < 2 {
            continue; // No total is needed
        }

        let sums: Vec<Option<f64>> = (key_count..width)
            .map(|col| sum_column(rows, &order[start..end_pos], col))
            .collect();

        if sums.iter().all(Option::is_none) {
            continue;
        }

        let mut row = vec![String::new(); width];
        row[level] = format!("{} {}", rows[order[start]][level], opts.total_label);
        for (col, sum) in (key_count..width).zip(sums) {
            if let Some(sum) = sum {
                row[col] = cell::format_total(sum);
            }
        }
        out.rows.push(row);
        out.row_kinds.push(OlapRowKind::Subtotal { level });
    }
}

/// Sums one column over the given rows, or `None` if nothing there is numeric.
fn sum_column(rows: &[Vec<String>], members: &[usize], col: usize) -> Option<f64> {
    let mut optional_sum = None;
    for &i in members {
        let cell = rows[i]
            .get(col)
            .map(String::as_str)
            .unwrap_or_default()
            .trim();
        if cell.is_empty() {
            continue;
        }
        optional_sum = Some(optional_sum.unwrap_or(0.0) + cell::parse_number(cell)?);
    }
    optional_sum
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn answer(data: Value) -> OlapAnswer {
        serde_json::from_value(json!({ "data": data })).unwrap()
    }

    fn fields(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn pivot_sums_duplicate_keys_into_one_row() {
        let table = answer(json!([
            { "Dish": "Tea", "Pay": "Cash", "Sum": 10 },
            { "Dish": "Tea", "Pay": "Cash", "Sum": 5 },
            { "Dish": "Tea", "Pay": "Card", "Sum": 2 },
        ]))
        .to_pivot_table(&fields(&["Dish"]), "Pay", "Sum", "Total");

        assert_eq!(table.columns, fields(&["Dish", "Card", "Cash"]));
        assert_eq!(table.key_count, 1);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0], fields(&["Tea", "2", "15"]));
        assert_eq!(table.rows[1], fields(&["Total", "2", "15"]));
        assert!(matches!(table.row_kinds[1], OlapRowKind::GrandTotal));
    }

    #[test]
    fn pivot_leaves_missing_combinations_blank() {
        let table = answer(json!([
            { "Dish": "Tea", "Pay": "Cash", "Sum": 10 },
            { "Dish": "Pie", "Pay": "Card", "Sum": 4 },
        ]))
        .to_pivot_table(&fields(&["Dish"]), "Pay", "Sum", "Total");

        assert_eq!(table.rows[0], fields(&["Pie", "4", ""]));
        assert_eq!(table.rows[1], fields(&["Tea", "", "10"]));
    }

    #[test]
    fn pivot_digs_into_nested_column_values() {
        let table = answer(json!([
            { "Dish": "Tea", "Pay": { "Cash": { "Sum": 3 }, "Card": { "Sum": 7 } } },
        ]))
        .to_pivot_table(&fields(&["Dish"]), "Pay", "Sum", "Total");

        assert_eq!(table.columns, fields(&["Dish", "Card", "Cash"]));
        assert_eq!(table.rows[0], fields(&["Tea", "7", "3"]));
    }

    #[test]
    fn grouped_blanks_repeated_keys_and_adds_subtotals() {
        let table = answer(json!([
            { "Shop": "North", "Dish": "Tea", "Sum": 10 },
            { "Shop": "North", "Dish": "Pie", "Sum": 5 },
            { "Shop": "South", "Dish": "Tea", "Sum": 3 },
        ]))
        .to_table_grouped(
            &fields(&["Shop", "Dish"]),
            GroupOptions::grouped("Total").with_grand_total(true),
        );

        let shop: Vec<&str> = table.rows.iter().map(|r| r[0].as_str()).collect();
        let dish: Vec<&str> = table.rows.iter().map(|r| r[1].as_str()).collect();

        assert_eq!(shop, ["North", "", "North Total", "South", "Total"]);
        assert_eq!(dish, ["Pie", "Tea", "", "Tea", ""]);

        assert!(matches!(
            table.row_kinds[2],
            OlapRowKind::Subtotal { level: 0 }
        ));
        assert!(matches!(table.row_kinds[4], OlapRowKind::GrandTotal));

        let sums: Vec<&str> = table.rows.iter().map(|r| r[2].as_str()).collect();
        assert_eq!(sums, ["5", "10", "15", "3", "18"]);
    }

    #[test]
    fn grouped_skips_subtotals_for_non_numeric_columns() {
        let table = answer(json!([
            { "Shop": "North", "Dish": "Tea", "Note": "hot" },
            { "Shop": "North", "Dish": "Pie", "Note": "cold" },
        ]))
        .to_table_grouped(&fields(&["Shop", "Dish"]), GroupOptions::grouped("Total"));

        assert_eq!(table.rows.len(), 2);
        assert!(
            table
                .row_kinds
                .iter()
                .all(|k| matches!(k, OlapRowKind::Data))
        );
    }

    #[test]
    fn empty_data_produces_an_empty_grouped_table() {
        let table =
            answer(json!([])).to_table_grouped(&fields(&["Shop"]), GroupOptions::grouped("Total"));

        assert!(table.rows.is_empty());
        assert!(table.row_kinds.is_empty());
    }
}
