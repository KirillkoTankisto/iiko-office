use std::sync::Arc;

use gtk4::glib;
use gtk4::{Button, prelude::*};

use crate::gui::main::menu::buttons::AnyButton;
use crate::gui::main::menu::tabs::open_tab;
use crate::gui::{
    GlobalData,
    main::menu::view::MainView,
    translation::{Line::CASH_SHIFTS, translate},
};

use crate::gui::main::menu::tabs::cashshifts::CashShiftsTab;

pub struct CashShiftsButton;

impl AnyButton for CashShiftsButton {
    fn attach_to(a_box: &gtk4::Box, gdata: Arc<GlobalData>, view: &MainView) {
        let button = Button::with_label(translate(gdata.language(), CASH_SHIFTS));

        button.connect_clicked(glib::clone!(
            #[strong]
            view,
            move |button| {
                button.set_sensitive(false);
                let widget = open_tab(&CashShiftsTab, gdata.clone(), &view);
                widget.connect_destroy(glib::clone!(
                    #[weak]
                    button,
                    move |_| button.set_sensitive(true)
                ));
            }
        ));

        a_box.append(&button);
    }
}
