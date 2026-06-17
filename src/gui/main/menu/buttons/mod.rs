use std::sync::Arc;

use gtk4::glib;

use gtk4::{Box, Button, Orientation::Vertical};

use gtk4::prelude::*;

use crate::gui::main::menu::tabs::{AnyTab, open_tab};
use crate::gui::{
    GlobalData,
    main::menu::{
        buttons::{cashshifts::CashShiftsButton, olap_reports::OlapReportsButton},
        view::MainView,
    },
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

pub fn create_any_button(
    anytab: &'static dyn AnyTab,
    label: &str,
    gdata: Arc<GlobalData>,
    view: &MainView,
) -> Button {
    let button = Button::with_label(label);

    button.connect_clicked(glib::clone!(
        #[strong]
        view,
        move |button| {
            button.set_sensitive(false);
            let widget = open_tab(anytab, gdata.clone(), &view);
            widget.connect_destroy(glib::clone!(
                #[weak]
                button,
                move |_| button.set_sensitive(true)
            ));
        }
    ));

    button
}

pub trait AnyButton {
    fn attach_to(a_box: &gtk4::Box, gdata: Arc<GlobalData>, view: &MainView);
}
