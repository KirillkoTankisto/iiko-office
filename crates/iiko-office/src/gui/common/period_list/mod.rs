use crate::gui::{
    common::dropdown::{AnyDropDown, DropDownItem},
    translation::{
        CurrentLanguage,
        Line::{self, *},
        translate,
    },
};
use iiko_api::consts::PeriodType;

pub struct PeriodList;

const PERIODS: &[PeriodType] = &[
    PeriodType::Custom,
    PeriodType::OpenPeriod,
    PeriodType::Today,
    PeriodType::Yesterday,
    PeriodType::CurrentWeek,
    PeriodType::CurrentMonth,
    PeriodType::CurrentYear,
    PeriodType::LastWeek,
    PeriodType::LastMonth,
    PeriodType::LastYear,
];

const fn period_line(period: PeriodType) -> Line {
    match period {
        PeriodType::Custom => PERIOD_CUSTOM,
        PeriodType::OpenPeriod => PERIOD_OPEN,
        PeriodType::Today => PERIOD_TODAY,
        PeriodType::Yesterday => PERIOD_YESTERDAY,
        PeriodType::CurrentWeek => PERIOD_CURRENT_WEEK,
        PeriodType::CurrentMonth => PERIOD_CURRENT_MONTH,
        PeriodType::CurrentYear => PERIOD_CURRENT_YEAR,
        PeriodType::LastWeek => PERIOD_LAST_WEEK,
        PeriodType::LastMonth => PERIOD_LAST_MONTH,
        PeriodType::LastYear => PERIOD_LAST_YEAR,
    }
}

impl DropDownItem for PeriodType {
    fn label(&self, language: CurrentLanguage) -> String {
        translate(language, period_line(*self)).to_string()
    }
}

impl PeriodList {
    pub fn build<U: Fn(Option<PeriodType>) + 'static>(
        language: CurrentLanguage,
        ui: U,
    ) -> AnyDropDown<PeriodType> {
        let dropdown = AnyDropDown::new(language, 180, PERIODS.to_vec());
        dropdown.connect_selected(ui);
        dropdown
    }
}
