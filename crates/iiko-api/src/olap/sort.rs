//! Ordering rules for table keys.
//!
//! Every cell reaches us as a string, but they do not all mean the same kind of
//! thing. Blanks sort first, then numbers, then dates, then text; within a kind
//! the comparison is the natural one rather than lexicographic, so `10` follows
//! `9` and `2026-01-02` follows `2026-01-01`.

use std::cmp::Ordering;

use crate::olap::cell;

/// Compares two cells by the kind of value they hold.
pub fn compare(a: &str, b: &str) -> Ordering {
    let (ka, kb) = (classify(a), classify(b));
    match (ka, kb) {
        (CellKey::Empty, CellKey::Empty) => Ordering::Equal,
        (CellKey::Number(x), CellKey::Number(y)) => x.total_cmp(&y),
        (CellKey::Date(x), CellKey::Date(y)) => x.cmp(&y),
        (CellKey::Text, CellKey::Text) => compare_ci(a, b),
        _ => ka.rank().cmp(&kb.rank()),
    }
}

/// Compares composite keys: the first difference wins, shorter key first.
pub fn compare_keys(a: &[String], b: &[String]) -> Ordering {
    a.iter()
        .zip(b)
        .map(|(x, y)| compare(x, y))
        .find(|o| o.is_ne())
        .unwrap_or_else(|| a.len().cmp(&b.len()))
}

/// Case-insensitive comparison, without allocating.
fn compare_ci(a: &str, b: &str) -> Ordering {
    a.chars()
        .flat_map(char::to_lowercase)
        .cmp(b.chars().flat_map(char::to_lowercase))
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
    /// Unlike values sort as blocks, in this order.
    fn rank(self) -> u8 {
        match self {
            CellKey::Empty => 0,
            CellKey::Number(_) => 1,
            CellKey::Date(_) => 2,
            CellKey::Text => 3,
        }
    }
}

fn classify(s: &str) -> CellKey {
    let t = s.trim();
    if t.is_empty() {
        CellKey::Empty
    } else if let Some(d) = parse_date(t) {
        CellKey::Date(d)
    } else if let Some(n) = cell::parse_number(t) {
        CellKey::Number(n)
    } else {
        CellKey::Text
    }
}

/// Parses `YYYY-MM-DD` or `DD.MM.YYYY`, optionally followed by a time.
/// Returns the fields as an array so they compare in the right order.
fn parse_date(s: &str) -> Option<[u16; 6]> {
    let s = s.trim_end_matches('Z');
    let (date, time) = match s.split_once(['T', ' ']) {
        Some((d, t)) => (d, Some(t)),
        None => (s, None),
    };

    let (y, m, d) = if let Some((a, rest)) = date.split_once('-') {
        let (b, c) = rest.split_once('-')?;
        (field(a, 4)?, field(b, 2)?, field(c, 2)?)
    } else if let Some((a, rest)) = date.split_once('.') {
        let (b, c) = rest.split_once('.')?;
        (field(c, 4)?, field(b, 2)?, field(a, 2)?)
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
            *out.get_mut(3 + i)? = field(part, 2)?;
        }
    }
    Some(out)
}

/// Parses one date field, rejecting anything that is not a short run of digits.
fn field(s: &str, max_len: usize) -> Option<u16> {
    if s.is_empty() || s.len() > max_len || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn iso_and_dotted_dates_both_parse() {
        assert_eq!(parse_date("2026-01-02"), Some([2026, 1, 2, 0, 0, 0]));
        assert_eq!(parse_date("02.01.2026"), Some([2026, 1, 2, 0, 0, 0]));
        assert_eq!(
            parse_date("2026-01-02T13:45:07.250Z"),
            Some([2026, 1, 2, 13, 45, 7])
        );
    }

    #[test]
    fn impossible_dates_are_rejected() {
        assert_eq!(parse_date("2026-13-01"), None);
        assert_eq!(parse_date("2026-01-32"), None);
        assert_eq!(parse_date("not-a-date"), None);
        assert_eq!(parse_date("2026/01/02"), None);
    }

    #[test]
    fn numbers_sort_numerically_not_lexicographically() {
        let mut values = keys(&["10", "9", "100", "2"]);
        values.sort_by(|a, b| compare(a, b));
        assert_eq!(values, keys(&["2", "9", "10", "100"]));
    }

    #[test]
    fn dates_sort_chronologically() {
        let mut values = keys(&["2026-02-01", "2026-01-31", "2025-12-31"]);
        values.sort_by(|a, b| compare(a, b));
        assert_eq!(values, keys(&["2025-12-31", "2026-01-31", "2026-02-01"]));
    }

    #[test]
    fn kinds_sort_as_blocks() {
        let mut values = keys(&["text", "2026-01-01", "5", ""]);
        values.sort_by(|a, b| compare(a, b));
        assert_eq!(values, keys(&["", "5", "2026-01-01", "text"]));
    }

    #[test]
    fn text_ignores_case() {
        assert_eq!(compare("apple", "APPLE"), Ordering::Equal);
        assert_eq!(compare("Apple", "banana"), Ordering::Less);
        // Cyrillic lowercases too.
        assert_eq!(compare("Ёлка", "ёлка"), Ordering::Equal);
    }

    #[test]
    fn composite_keys_break_on_first_difference() {
        assert_eq!(
            compare_keys(&keys(&["a", "2"]), &keys(&["a", "10"])),
            Ordering::Less
        );
        assert_eq!(
            compare_keys(&keys(&["b", "1"]), &keys(&["a", "99"])),
            Ordering::Greater
        );
        // A prefix sorts before the longer key it prefixes.
        assert_eq!(
            compare_keys(&keys(&["a"]), &keys(&["a", "1"])),
            Ordering::Less
        );
    }
}
