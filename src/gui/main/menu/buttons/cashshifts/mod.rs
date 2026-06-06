use std::sync::Arc;

use gtk4::{Button, Notebook, prelude::*};

use crate::gui::{
    GlobalData, main::menu::tabs::cashshifts::create_cashshifts, translation::{Line::CASH_SHIFTS, translate}
};

pub fn create_cashshifts_button(gdata: Arc<GlobalData>, view: &Notebook) -> Button {
    let button = Button::with_label(translate(gdata.language.clone(), CASH_SHIFTS));
    let view = view.clone();
    button.connect_clicked(move |_| {
        create_cashshifts(gdata.clone(), &view);
    });
    button
}
