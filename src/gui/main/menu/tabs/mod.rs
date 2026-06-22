use std::sync::Arc;

use gtk4::{Button, Orientation::{Horizontal, Vertical}};

use crate::gui::{GlobalData, main::menu::view::MainView};

pub mod cashshifts;
pub mod cashshifts_payments;
pub mod olap_reports;

pub trait AnyTab {
    fn title(&self, gdata: &GlobalData) -> &str;

    fn build(&self, gdata: Arc<GlobalData>, view: &MainView) -> gtk4::Widget;
}

pub fn open_tab(tab: &dyn AnyTab, gdata: Arc<GlobalData>, view: &MainView, button: Option<&Button>) {
    view.add_tab(tab, gdata, view, button);
}

pub fn build_box() -> gtk4::Box {
    gtk4::Box::builder()
        .orientation(Vertical)
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build()
}

pub fn build_hbox() -> gtk4::Box {
    gtk4::Box::builder()
    .orientation(Horizontal)
    .spacing(8)
    .margin_start(8)
    .margin_end(8)
    .margin_top(8)
    .margin_bottom(8)
    .build()
}
