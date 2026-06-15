use std::sync::Arc;

use gtk4::glib;
use gtk4::{Button, prelude::*};

use crate::gui::main::menu::tabs::open_tab;
use crate::gui::{
    GlobalData,
    main::menu::view::MainView,
    translation::{Line::CASH_SHIFTS, translate},
};

use crate::gui::main::menu::tabs::cashshifts::CashShiftsTab;

pub struct CashShiftsButton {
    button: Button,
}

impl CashShiftsButton {
    pub fn new(gdata: Arc<GlobalData>, view: &MainView) -> Self {
        let button = Button::with_label(translate(gdata.language(), CASH_SHIFTS));

        button.connect_clicked(glib::clone!(
            #[strong]
            view,
            move |_| {
                open_tab(&CashShiftsTab, gdata.clone(), &view);
            }
        ));

        Self { button }
    }

    pub fn present(&self) -> &Button {
        &self.button
    }
}
