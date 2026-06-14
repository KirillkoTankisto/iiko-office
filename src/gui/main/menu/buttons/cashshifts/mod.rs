use std::sync::Arc;

use gtk4::{Button, prelude::*};
use gtk4::glib;

use crate::gui::{
    GlobalData,
    main::menu::{tabs::cashshifts::create_cashshifts, view::MainView},
    translation::{Line::CASH_SHIFTS, translate},
};

pub fn create_cashshifts_button(gdata: Arc<GlobalData>, view: &MainView) -> Button {
    let button = Button::with_label(translate(gdata.language, CASH_SHIFTS));

    button.connect_clicked(glib::clone!(
        #[strong]
        view,
        move |_| {
        create_cashshifts(gdata.clone(), view.clone());
    }));

    button
}
