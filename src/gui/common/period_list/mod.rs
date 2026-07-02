use crate::{api::consts::PeriodType, gui::translation::CurrentLanguage};
use gtk4::glib;

#[derive(glib::Downgrade)]
pub struct PeriodList {
    root: gtk4::DropDown,
    language: CurrentLanguage,
}

const ALL_PERIODS: &[PeriodType] = &[
    PeriodType::CUSTOM,
    PeriodType::OPEN_PERIOD,
    PeriodType::TODAY,
    PeriodType::YESTERDAY,
    PeriodType::CURRENT_WEEK,
    PeriodType::CURRENT_MONTH,
    PeriodType::CURRENT_YEAR,
    PeriodType::LAST_WEEK,
    PeriodType::LAST_MONTH,
    PeriodType::LAST_YEAR,
];

impl PeriodList {
    pub fn new<U: Fn(bool) + 'static>(language: CurrentLanguage, ui: U) -> Self {
        let titles: Vec<&str> = ALL_PERIODS.iter().map(|p| p.title(language)).collect();
        let list_model = gtk4::StringList::new(&titles);

        let root = gtk4::DropDown::builder()
            .model(&list_model)
            .selected(0)
            .width_request(180)
            .build();

        root.connect_selected_notify(move |dropdown| {
            let is_custom = ALL_PERIODS
                .get(dropdown.selected() as usize)
                .map(|p| *p as usize == PeriodType::CUSTOM as usize)
                .unwrap_or(false);
            ui(is_custom);
        });

        Self { root, language }
    }

    pub fn get_value(&self) -> PeriodType {
        *ALL_PERIODS
            .get(self.root.selected() as usize)
            .unwrap_or(&PeriodType::CUSTOM)
    }

    pub fn present(&self) -> &gtk4::DropDown {
        &self.root
    }
}
