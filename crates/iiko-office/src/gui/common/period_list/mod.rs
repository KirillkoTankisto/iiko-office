use crate::gui::translation::{CurrentLanguage, Line, Line::*, translate};
use gtk4::glib;
use iiko_api::consts::PeriodType;

#[derive(glib::Downgrade)]
pub struct PeriodList {
    root: gtk4::DropDown,
}

const PERIODS: &[(PeriodType, Line)] = &[
    (PeriodType::Custom, PERIOD_CUSTOM),
    (PeriodType::OpenPeriod, PERIOD_OPEN),
    (PeriodType::Today, PERIOD_TODAY),
    (PeriodType::Yesterday, PERIOD_YESTERDAY),
    (PeriodType::CurrentWeek, PERIOD_CURRENT_WEEK),
    (PeriodType::CurrentMonth, PERIOD_CURRENT_MONTH),
    (PeriodType::CurrentYear, PERIOD_CURRENT_YEAR),
    (PeriodType::LastWeek, PERIOD_LAST_WEEK),
    (PeriodType::LastMonth, PERIOD_LAST_MONTH),
    (PeriodType::LastYear, PERIOD_LAST_YEAR),
];

impl PeriodList {
    pub fn new<U: Fn(bool) + 'static>(language: CurrentLanguage, ui: U) -> Self {
        let titles: Vec<&str> = PERIODS
            .iter()
            .map(|(_, line)| translate(language, *line))
            .collect();
        let list_model = gtk4::StringList::new(&titles);

        let root = gtk4::DropDown::builder()
            .model(&list_model)
            .selected(0)
            .width_request(180)
            .build();

        root.connect_selected_notify(move |dropdown| {
            ui(Self::period_at(dropdown.selected()) == PeriodType::Custom);
        });

        Self { root }
    }

    fn period_at(index: u32) -> PeriodType {
        PERIODS
            .get(index as usize)
            .map_or(PeriodType::Custom, |(period, _)| *period)
    }

    pub fn get_value(&self) -> PeriodType {
        Self::period_at(self.root.selected())
    }

    pub fn present(&self) -> &gtk4::DropDown {
        &self.root
    }
}
