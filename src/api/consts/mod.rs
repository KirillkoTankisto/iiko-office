use serde::Serialize;

use crate::gui::translation::{
    CurrentLanguage,
    Line::{
        PERIOD_CURRENT_MONTH, PERIOD_CURRENT_WEEK, PERIOD_CURRENT_YEAR, PERIOD_CUSTOM,
        PERIOD_LAST_MONTH, PERIOD_LAST_WEEK, PERIOD_LAST_YEAR, PERIOD_OPEN, PERIOD_TODAY,
        PERIOD_YESTERDAY,
    },
    translate,
};

#[derive(Serialize)]
pub enum FilterType {
    IncludeValues,
    ExcludeValues,
    DateRange,
}

#[allow(nonstandard_style)]
#[derive(Clone, Copy, Serialize)]
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

impl PeriodType {
    pub fn title(&self, language: CurrentLanguage) -> &str {
        match self {
            PeriodType::CUSTOM => translate(language, PERIOD_CUSTOM),
            PeriodType::OPEN_PERIOD => translate(language, PERIOD_OPEN),
            PeriodType::TODAY => translate(language, PERIOD_TODAY),
            PeriodType::YESTERDAY => translate(language, PERIOD_YESTERDAY),
            PeriodType::CURRENT_WEEK => translate(language, PERIOD_CURRENT_WEEK),
            PeriodType::CURRENT_MONTH => translate(language, PERIOD_CURRENT_MONTH),
            PeriodType::CURRENT_YEAR => translate(language, PERIOD_CURRENT_YEAR),
            PeriodType::LAST_WEEK => translate(language, PERIOD_LAST_WEEK),
            PeriodType::LAST_MONTH => translate(language, PERIOD_LAST_MONTH),
            PeriodType::LAST_YEAR => translate(language, PERIOD_LAST_YEAR),
        }
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum EnumRange {
    I64(i64),
    F64(f64),
    String(String),
}
