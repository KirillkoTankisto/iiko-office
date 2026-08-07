//! Turning JSON values into the text and numbers a table cell holds.

use indexmap::IndexMap;
use serde_json::Value;

/// Renders a value for display: numbers are tidied up, arrays are joined.
pub fn render(value: &Value) -> String {
    match value {
        Value::Number(n) => n.as_f64().map_or_else(|| n.to_string(), format_number),
        Value::Array(arr) => arr.iter().map(render).collect::<Vec<_>>().join(", "),
        other => text(other),
    }
}

/// Stringifies a value without reformatting numbers. `null` becomes blank.
pub fn text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Reads a value as a number. Anything unparsable counts as zero.
pub fn number(value: &Value) -> f64 {
    match value {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.trim().replace(',', ".").parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Formats an aggregate, leaving the cell blank when it is exactly zero.
pub fn format_total(value: f64) -> String {
    if value == 0.0 {
        String::new()
    } else {
        format_number(value)
    }
}

/// Formats a number, dropping a negligible fractional part.
pub fn format_number(value: f64) -> String {
    if value.fract().abs() < 1e-9 {
        (value.round() as i64).to_string()
    } else {
        format!("{value:.2}")
    }
}

/// Parses a rendered cell back into a number, accepting `,` as the decimal
/// separator. Returns `None` for anything that does not start like a number.
pub fn parse_number(cell: &str) -> Option<f64> {
    let first = *cell.as_bytes().first()?;
    if !(first.is_ascii_digit() || first == b'-' || first == b'+') {
        return None;
    }
    let n: f64 = cell.replace(',', ".").parse().ok()?;
    n.is_finite().then_some(n)
}

/// Expands nested objects and arrays into `prefix / key` and `prefix[i]`
/// columns, so a record of any shape becomes a flat row.
pub fn flatten(prefix: &str, value: &Value, out: &mut IndexMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                flatten(&format!("{prefix} / {key}"), value, out);
            }
        }
        Value::Array(arr) if arr.iter().any(|v| v.is_object() || v.is_array()) => {
            for (i, item) in arr.iter().enumerate() {
                flatten(&format!("{prefix}[{i}]"), item, out);
            }
        }
        scalar => {
            out.insert(prefix.to_string(), render(scalar));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn whole_numbers_lose_their_decimals() {
        assert_eq!(format_number(12.0), "12");
        assert_eq!(format_number(-3.000000000001), "-3");
        assert_eq!(format_number(12.5), "12.50");
        assert_eq!(format_number(12.567), "12.57");
    }

    #[test]
    fn zero_totals_render_blank() {
        assert_eq!(format_total(0.0), "");
        assert_eq!(format_total(1.0), "1");
    }

    #[test]
    fn null_becomes_blank_and_strings_stay_verbatim() {
        assert_eq!(text(&Value::Null), "");
        assert_eq!(text(&json!("  padded  ")), "  padded  ");
        assert_eq!(text(&json!(true)), "true");
    }

    #[test]
    fn arrays_are_joined() {
        assert_eq!(render(&json!(["a", "b"])), "a, b");
        assert_eq!(render(&json!([1.0, 2.5])), "1, 2.50");
    }

    #[test]
    fn numbers_accept_comma_decimals() {
        assert_eq!(number(&json!("1,5")), 1.5);
        assert_eq!(number(&json!(2.25)), 2.25);
        assert_eq!(number(&json!("not a number")), 0.0);
        assert_eq!(number(&Value::Null), 0.0);
    }

    #[test]
    fn parse_number_rejects_text() {
        assert_eq!(parse_number("42"), Some(42.0));
        assert_eq!(parse_number("-1,5"), Some(-1.5));
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("12 items"), None);
        // Leading letters disqualify it even if a number follows.
        assert_eq!(parse_number("x1"), None);
        // Infinity is not a usable total.
        assert_eq!(parse_number("inf"), None);
    }

    #[test]
    fn nested_objects_become_prefixed_columns() {
        let mut out = IndexMap::new();
        flatten(
            "Sales",
            &json!({ "Cash": 10, "Card": { "Visa": 5 } }),
            &mut out,
        );

        assert_eq!(out.get("Sales / Cash").map(String::as_str), Some("10"));
        assert_eq!(
            out.get("Sales / Card / Visa").map(String::as_str),
            Some("5")
        );
    }

    #[test]
    fn scalar_arrays_stay_in_one_column() {
        let mut out = IndexMap::new();
        flatten("Tags", &json!(["a", "b"]), &mut out);
        assert_eq!(out.get("Tags").map(String::as_str), Some("a, b"));

        // ...but arrays of objects are split apart.
        let mut out = IndexMap::new();
        flatten("Rows", &json!([{ "n": 1 }, { "n": 2 }]), &mut out);
        assert_eq!(out.get("Rows[0] / n").map(String::as_str), Some("1"));
        assert_eq!(out.get("Rows[1] / n").map(String::as_str), Some("2"));
    }
}
