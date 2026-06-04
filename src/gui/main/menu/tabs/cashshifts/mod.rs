use gtk4::{Box, Notebook, prelude::*};

use crate::gui::{
    common::datepicker::DatePicker, main::menu::tabs::add_tab, translation::{CurrentLanguage, Line::{CASH_SHIFTS, DATE_FROM, DATE_TO}, translate},
};

pub fn create_cashshifts(view: &Notebook, language: &CurrentLanguage) {
    let cashshifts_box = Box::builder()
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let grid = gtk4::Grid::builder()
        .column_spacing(8)
        .row_spacing(8)
        .build();

    let date_from = DatePicker::new(translate(language.clone(), DATE_FROM), language);
    let date_to = DatePicker::new(translate(language.clone(), DATE_TO), language);

    date_from.attach_to(&grid, 0);
    date_to.attach_to(&grid, 1);

    cashshifts_box.append(&grid);

    view.append_page(
        &cashshifts_box,
        Some(&add_tab(view, &cashshifts_box, translate(language.clone(), CASH_SHIFTS))),
    );
}
