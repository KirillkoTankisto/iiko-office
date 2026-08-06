use std::sync::Arc;

use gtk4::glib;

use gtk4::{Box, Button, Orientation::Vertical};

use gtk4::prelude::*;

use crate::gui::main::menu::tabs::AnyTab;
use crate::gui::main::menu::tabs::cashshifts::CashShiftsTab;
use crate::gui::main::menu::tabs::olap_reports::OlapReportsTab;
use crate::gui::translation::{Line, translate};
use crate::gui::{GlobalData, main::menu::view::MainView};

/// Every entry becomes one sidebar button that opens the associated tab.
const TAB_BUTTONS: &[(&dyn AnyTab, Line)] = &[
    (&CashShiftsTab, Line::CASH_SHIFTS),
    (&OlapReportsTab, Line::OLAP_REPORTS),
];

pub fn create_buttons(gdata: Arc<GlobalData>, view: &MainView) -> Box {
    let buttons_box = Box::builder()
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_bottom(8)
        .margin_top(8)
        .orientation(Vertical)
        .build();

    for (tab, line) in TAB_BUTTONS {
        buttons_box.append(&create_any_button(
            *tab,
            translate(gdata.language(), *line),
            gdata.clone(),
            view,
        ));
    }

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
        #[weak]
        gdata,
        move |button| {
            button.set_sensitive(false);
            view.add_tab(anytab, gdata, Some(button));
        }
    ));

    button
}
