use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{Button, glib};

use crate::gui::main::menu::tabs::olap_reports::OlapReportsTab;
use crate::gui::main::menu::tabs::open_tab;
use crate::gui::translation::Line::OLAP_REPORTS;
use crate::gui::translation::translate;
use crate::gui::{
    GlobalData,
    main::menu::{buttons::AnyButton, view::MainView},
};

pub struct OlapReportsButton;

impl AnyButton for OlapReportsButton {
    fn attach_to(a_box: &gtk4::Box, gdata: Arc<GlobalData>, view: &MainView) {
        let button = Button::with_label(translate(gdata.language(), OLAP_REPORTS));

        button.connect_clicked(glib::clone!(
            #[strong]
            view,
            move |button| {
                button.set_sensitive(false);
                let widget = open_tab(&OlapReportsTab, gdata.clone(), &view);
                widget.connect_destroy(glib::clone!(
                    #[weak]
                    button,
                    move |_| { button.set_sensitive(true) }
                ));
            }
        ));
        a_box.append(&button);
    }
}
