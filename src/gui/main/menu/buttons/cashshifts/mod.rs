use gtk4::{Button, Notebook, prelude::*};

use crate::gui::{main::menu::tabs::cashshifts::create_cashshifts, translation::{CurrentLanguage, Line::CASH_SHIFTS, translate}};

pub fn create_cashshifts_button<'a>(view: &'a Notebook, language: &'a CurrentLanguage) -> Button {
    let button = Button::with_label(translate(language.clone(), CASH_SHIFTS));

    let view = view.clone();
    let language = language.clone();
    button.connect_clicked(move |_| {
        create_cashshifts(&view, &language);
    });

    button
}
