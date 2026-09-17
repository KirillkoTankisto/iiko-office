use std::sync::Arc;

use gtk4::{Box, Button, glib, prelude::*};

use crate::gui::common::anybox::AnyBox;
use crate::gui::main::menu::tabs::AnyTab;
use crate::gui::main::menu::tabs::cashshifts::CashShiftsTab;
use crate::gui::main::menu::tabs::employees::EmployeesTab;
use crate::gui::main::menu::tabs::olap_reports::OlapReportsTab;
use crate::gui::translation::{Line, translate};
use crate::gui::{GlobalData, main::menu::view::MainView};

/// Every entry becomes a sidebar button that opens the associated tab.
const TAB_BUTTONS: &[(&dyn AnyTab, Line)] = &[
    (&CashShiftsTab, Line::CASH_SHIFTS),
    (&OlapReportsTab, Line::OLAP_REPORTS),
    (&EmployeesTab, Line::EMPLOYEES),
];

pub fn create_buttons(gdata: Arc<GlobalData>, view: &MainView) -> Box {
    let abox = AnyBox::vertical().margin(8);

    for (tab, line) in TAB_BUTTONS {
        abox.add(&create_any_button(
            *tab,
            translate(gdata.language(), *line),
            gdata.clone(),
            view,
        ));
    }

    abox.consume()
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
