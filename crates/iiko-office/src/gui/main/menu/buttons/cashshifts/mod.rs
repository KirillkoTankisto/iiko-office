use std::sync::Arc;

use gtk4::prelude::*;

use crate::gui::main::menu::buttons::{AnyButton, create_any_button};
use crate::gui::{
    GlobalData,
    main::menu::view::MainView,
    translation::{Line::CASH_SHIFTS, translate},
};

use crate::gui::main::menu::tabs::cashshifts::CashShiftsTab;

pub struct CashShiftsButton;

impl AnyButton for CashShiftsButton {
    fn attach_to(a_box: &gtk4::Box, gdata: Arc<GlobalData>, view: &MainView) {
        let button = create_any_button(
            &CashShiftsTab,
            translate(gdata.language(), CASH_SHIFTS),
            gdata,
            view,
        );

        a_box.append(&button);
    }
}
