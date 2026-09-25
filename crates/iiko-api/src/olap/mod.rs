//! OLAP отчёты: формирование, получение и преобразование в таблицу

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};

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
/// Запрос OLAP
pub struct OlapRequest {
    pub report_type: ReportType,
    pub build_summary: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub group_by_row_fields: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub group_by_col_fields: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub aggregate_fields: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub filters: FilterMap,
}

/// Таблица фильтров
pub type FilterMap = BTreeMap<String, Filter>;

#[derive(Serialize)]
#[serde(tag = "filterType", rename_all_fields = "camelCase")]
/// Фильтр для OLAP отчёта
pub enum Filter {
    /// Включить значения
    IncludeValues { values: Vec<String> },
    /// Исключить значения
    ExcludeValues { values: Vec<String> },
    /// Выбрать временной промежуток
    DateRange {
        period_type: PeriodType,
        from: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        to: Option<String>,
    },
    /// Выбрать промежуток значений
    ValueRange {
        from: EnumRange,
        to: EnumRange,
        include_low: bool,
        include_high: bool,
    },
}

impl Filter {
    /// ID поля с фильтром даты
    pub const OPEN_DATE_FIELD: &str = "OpenDate.Typed";

    /// Выбрать свой промежуток даты
    pub fn custom_date_range(from: String, to: String) -> Self {
        Self::DateRange {
            period_type: PeriodType::Custom,
            from,
            to: Some(to),
        }
    }

    /// Выбрать предустановленный промежуток даты
    pub fn preset_date_range(period_type: PeriodType) -> Self {
        Self::DateRange {
            period_type,
            from: "2000-01-01T00:00:00.000".into(), // Заглушка, не трогать
            to: None,
        }
    }

    /// Выбрать промежуток значений
    pub fn closed_value_range(from: EnumRange, to: EnumRange) -> Self {
        Self::ValueRange {
            from,
            to,
            include_low: true,
            include_high: true,
        }
    }
}

/// Блок с итоговыми суммами
pub type SummaryBlock = Vec<IndexMap<String, String>>;

#[derive(Deserialize, Debug)]
/// Ответ с OLAP отчётом
pub struct OlapAnswer {
    pub data: Vec<IndexMap<String, Value>>,
    #[serde(default)]
    pub summary: Vec<SummaryBlock>,
}

impl IikoSession {
    /// OLAP отчёт
    pub fn olap(&self, request: &OlapRequest) -> Result<OlapAnswer, ClientError> {
        let body = serde_json::to_string(request)?;
        self.request_post("/resto/api/v2/reports/olap", &[], body)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Тип строки
pub enum RowKind {
    Data,
    Subtotal,
    Total,
}

#[derive(Debug)]
/// Строка
pub struct Row {
    pub kind: RowKind,
    pub cells: Vec<String>,
}

#[derive(Debug)]
/// Готовая таблица с отчётом
pub struct OlapTable {
    pub columns: Vec<String>,
    pub rows: Vec<Row>,
    /// Количество столбцов с ключами группировки
    pub key_count: usize,
}

impl OlapAnswer {
    /// Построить кросс-таблицу
    pub fn to_cross_table(
        &self,
        row_fields: &[String],
        col_field: &str,
        value_field: &str,
        total_label: &str,
    ) -> OlapTable {
        let mut grid: BTreeMap<Vec<String>, HashMap<String, f64>> = BTreeMap::new();
        for record in &self.data {
            let Some(value) = record.get(col_field) else {
                continue;
            };
            let key = row_fields
                .iter()
                .map(|f| record.get(f).map(text).unwrap_or_default())
                .collect();
            let sums = grid.entry(key).or_default();

            match value {
                Value::Object(map) => {
                    for (col, v) in map {
                        *sums.entry(col.clone()).or_default() +=
                            number(v.get(value_field).unwrap_or(v));
                    }
                }
                other => {
                    *sums.entry(text(other)).or_default() +=
                        record.get(value_field).map_or(0.0, number);
                }
            }
        }

        let headers: BTreeSet<&String> = grid.values().flat_map(HashMap::keys).collect();
        let mut headers: Vec<String> = headers.into_iter().cloned().collect();
        headers.sort_by(|a, b| compare(a, b));

        let key_count = row_fields.len();
        let mut totals = vec![0.0; headers.len()];
        let mut rows: Vec<Row> = grid
            .into_iter()
            .map(|(mut cells, sums)| {
                for (col, total) in headers.iter().zip(&mut totals) {
                    let n = sums.get(col).copied().unwrap_or(0.0);
                    *total += n;
                    cells.push(fmt_total(n));
                }
                Row {
                    kind: RowKind::Data,
                    cells,
                }
            })
            .collect();
        rows.sort_by(|a, b| compare_keys(&a.cells[..key_count], &b.cells[..key_count]));

        let mut cells = vec![String::new(); key_count];
        if let Some(first) = cells.first_mut() {
            *first = total_label.to_string();
        }
        cells.extend(totals.into_iter().map(fmt_total));
        rows.push(Row {
            kind: RowKind::Total,
            cells,
        });

        OlapTable {
            columns: row_fields.iter().chain(&headers).cloned().collect(),
            rows,
            key_count,
        }
    }

    /// Построить сгруппированную таблицу
    pub fn to_grouped_table(&self, row_fields: &[String], total_label: &str) -> OlapTable {
        let mut all = IndexSet::new();
        let mut records = Vec::with_capacity(self.data.len());
        for record in &self.data {
            let mut flat = IndexMap::new();
            for (key, value) in record {
                flatten(key, value, &mut flat);
            }
            all.extend(flat.keys().cloned());
            records.push(flat);
        }

        let mut columns: IndexSet<String> = row_fields
            .iter()
            .filter_map(|f| all.iter().find(|c| c.starts_with(f.as_str())).cloned())
            .collect();
        let key_count = columns.len();
        columns.extend(all);

        let mut grid: Vec<Vec<String>> = records
            .into_iter()
            .map(|mut flat| {
                columns
                    .iter()
                    .map(|c| flat.swap_remove(c).unwrap_or_default())
                    .collect()
            })
            .collect();
        grid.sort_by(|a, b| compare_keys(&a[..key_count], &b[..key_count]));

        let mut rows = Vec::new();
        group(&grid, 0, key_count, total_label, &mut rows);
        if let Some(cells) = total_row(&grid, key_count, 0, total_label) {
            rows.push(Row {
                kind: RowKind::Total,
                cells,
            });
        }

        OlapTable {
            columns: columns.into_iter().collect(),
            rows,
            key_count,
        }
    }
}

fn group(grid: &[Vec<String>], level: usize, key_count: usize, label: &str, out: &mut Vec<Row>) {
    if level == key_count {
        out.extend(grid.iter().map(|cells| Row {
            kind: RowKind::Data,
            cells: cells.clone(),
        }));
        return;
    }

    for chunk in grid.chunk_by(|a, b| a[level] == b[level]) {
        let start = out.len();
        group(chunk, level + 1, key_count, label, out);

        for row in &mut out[start + 1..] {
            row.cells[level].clear();
        }

        if chunk.len() > 1 && level + 1 < key_count {
            let label = format!("{} {label}", chunk[0][level]);
            if let Some(cells) = total_row(chunk, key_count, level, &label) {
                out.push(Row {
                    kind: RowKind::Subtotal,
                    cells,
                });
            }
        }
    }
}

fn total_row(
    grid: &[Vec<String>],
    key_count: usize,
    level: usize,
    label: &str,
) -> Option<Vec<String>> {
    let mut cells = vec![String::new(); grid.first()?.len()];
    cells[level] = label.to_string();

    let mut numeric = false;
    for (col, cell) in cells.iter_mut().enumerate().skip(key_count) {
        let sum = grid
            .iter()
            .filter_map(|row| parse_number(&row[col]))
            .reduce(|a, b| a + b);
        if let Some(sum) = sum {
            *cell = fmt_total(sum);
            numeric = true;
        }
    }
    numeric.then_some(cells)
}

fn flatten(prefix: &str, value: &Value, out: &mut IndexMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                flatten(&format!("{prefix} / {key}"), value, out);
            }
        }
        scalar => {
            out.insert(prefix.to_string(), text(scalar));
        }
    }
}

fn text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.as_f64().map_or_else(|| n.to_string(), fmt),
        Value::Array(arr) => arr.iter().map(text).collect::<Vec<_>>().join(", "),
        other => other.to_string(),
    }
}

fn number(value: &Value) -> f64 {
    match value {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => parse_number(s).unwrap_or(0.0),
        _ => 0.0,
    }
}

fn parse_number(cell: &str) -> Option<f64> {
    cell.trim()
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|n| n.is_finite())
}

fn fmt(n: f64) -> String {
    let mut s = format!("{n:.2}");
    if s.ends_with(".00") {
        s.truncate(s.len() - 3);
    }
    s
}

fn fmt_total(n: f64) -> String {
    if n == 0.0 { String::new() } else { fmt(n) }
}

fn compare(a: &str, b: &str) -> Ordering {
    match (parse_number(a), parse_number(b)) {
        (Some(x), Some(y)) => x.total_cmp(&y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

fn compare_keys(a: &[String], b: &[String]) -> Ordering {
    a.iter()
        .zip(b)
        .map(|(x, y)| compare(x, y))
        .find(|o| o.is_ne())
        .unwrap_or_else(|| a.len().cmp(&b.len()))
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

    fn column(table: &OlapTable, i: usize) -> Vec<&str> {
        table.rows.iter().map(|r| r.cells[i].as_str()).collect()
    }

    #[test]
    fn pivot_sums_repeated_keys_and_totals_the_columns() {
        let table = answer(json!([
            { "Dish": "Tea", "Pay": "Cash", "Sum": 10 },
            { "Dish": "Tea", "Pay": "Cash", "Sum": 5 },
            { "Dish": "Pie", "Pay": "Card", "Sum": 2 },
        ]))
        .to_cross_table(&fields(&["Dish"]), "Pay", "Sum", "Total");

        assert_eq!(table.columns, fields(&["Dish", "Card", "Cash"]));
        assert_eq!(column(&table, 0), ["Pie", "Tea", "Total"]);
        assert_eq!(column(&table, 1), ["2", "", "2"]);
        assert_eq!(column(&table, 2), ["", "15", "15"]);
    }

    #[test]
    fn pivot_digs_into_nested_column_values() {
        let table = answer(json!([
            { "Dish": "Tea", "Pay": { "Cash": { "Sum": 3 }, "Card": { "Sum": 7 } } },
        ]))
        .to_cross_table(&fields(&["Dish"]), "Pay", "Sum", "Total");

        assert_eq!(table.columns, fields(&["Dish", "Card", "Cash"]));
        assert_eq!(table.rows[0].cells, fields(&["Tea", "7", "3"]));
    }

    #[test]
    fn grouped_blanks_repeats_and_adds_totals() {
        let table = answer(json!([
            { "Shop": "North", "Dish": "Tea", "Sum": 10 },
            { "Shop": "North", "Dish": "Pie", "Sum": 5 },
            { "Shop": "South", "Dish": "Tea", "Sum": 3 },
        ]))
        .to_grouped_table(&fields(&["Shop", "Dish"]), "Total");

        assert_eq!(
            column(&table, 0),
            ["North", "", "North Total", "South", "Total"]
        );
        assert_eq!(column(&table, 1), ["Pie", "Tea", "", "Tea", ""]);
        assert_eq!(column(&table, 2), ["5", "10", "15", "3", "18"]);
        assert_eq!(table.rows[2].kind, RowKind::Subtotal);
        assert_eq!(table.rows[4].kind, RowKind::Total);
    }

    #[test]
    fn grouped_skips_totals_when_nothing_is_numeric() {
        let table = answer(json!([
            { "Shop": "North", "Dish": "Tea", "Note": "hot" },
            { "Shop": "North", "Dish": "Pie", "Note": "cold" },
        ]))
        .to_grouped_table(&fields(&["Shop", "Dish"]), "Total");

        assert_eq!(table.rows.len(), 2);
        assert!(table.rows.iter().all(|r| r.kind == RowKind::Data));
    }

    #[test]
    fn nested_objects_become_their_own_columns() {
        let table = answer(json!([{ "Shop": "North", "Sales": { "Cash": 10 } }]))
            .to_grouped_table(&fields(&["Shop"]), "Total");

        assert_eq!(table.columns, fields(&["Shop", "Sales / Cash"]));
        assert_eq!(table.rows[0].cells, fields(&["North", "10"]));
    }

    #[test]
    fn empty_data_gives_an_empty_table() {
        let table = answer(json!([])).to_grouped_table(&fields(&["Shop"]), "Total");
        assert!(table.rows.is_empty());
    }

    #[test]
    fn numbers_sort_by_value_and_before_text() {
        let mut values = fields(&["text", "10", "9", ""]);
        values.sort_by(|a, b| compare(a, b));
        assert_eq!(values, fields(&["9", "10", "", "text"]));
        assert_eq!(compare("apple", "APPLE"), Ordering::Equal);
    }

    #[test]
    fn numbers_lose_a_pointless_fraction() {
        assert_eq!(fmt(12.0), "12");
        assert_eq!(fmt(12.001), "12");
        assert_eq!(fmt(12.567), "12.57");
        assert_eq!(fmt_total(0.0), "");
        assert_eq!(number(&json!("1,5")), 1.5);
        assert_eq!(parse_number("12 items"), None);
    }
}
