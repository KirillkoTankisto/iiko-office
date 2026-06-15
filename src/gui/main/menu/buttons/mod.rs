use std::sync::Arc;

use gtk4::{Box, Orientation::Vertical};

use crate::gui::{
    GlobalData,
    main::menu::{buttons::{cashshifts::CashShiftsButton, olap_reports::OlapReportsButton}, view::MainView},
};

mod cashshifts;
mod olap_reports;

pub fn create_buttons(gdata: Arc<GlobalData>, view: &MainView) -> Box {
    let buttons_box = Box::builder()
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_bottom(8)
        .margin_top(8)
        .orientation(Vertical)
        .build();

    CashShiftsButton::attach_to(&buttons_box, gdata.clone(), view);
    OlapReportsButton::attach_to(&buttons_box, gdata.clone(), view);

    buttons_box
}

pub trait AnyButton {
    fn attach_to(a_box: &gtk4::Box, gdata: Arc<GlobalData>, view: &MainView);
}
