use std::sync::Arc;

use gtk4::{Box, Orientation::Vertical, prelude::BoxExt};

use crate::gui::{
    GlobalData,
    main::menu::{buttons::cashshifts::CashShiftsButton, view::MainView},
};

mod cashshifts;

pub fn create_buttons(gdata: Arc<GlobalData>, view: &MainView) -> Box {
    let buttons_box = Box::builder()
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_bottom(8)
        .margin_top(8)
        .orientation(Vertical)
        .build();

    buttons_box.append(CashShiftsButton::new(gdata, view).present());

    buttons_box
}
