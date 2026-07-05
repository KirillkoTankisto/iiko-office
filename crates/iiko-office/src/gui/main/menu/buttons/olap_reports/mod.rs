use std::sync::Arc;

use gtk4::prelude::*;

use crate::gui::main::menu::buttons::create_any_button;
use crate::gui::main::menu::tabs::olap_reports::OlapReportsTab;
use crate::gui::translation::Line::OLAP_REPORTS;
use crate::gui::translation::translate;
use crate::gui::{
    GlobalData,
    main::menu::{buttons::AnyButton, view::MainView},
};

pub struct OlapReportsButton;

impl AnyButton for OlapReportsButton {
    fn attach_to(a_box: &gtk4::Box, gdata: Arc<GlobalData>, view: &MainView) {
        let button = create_any_button(
            &OlapReportsTab,
            translate(gdata.language(), OLAP_REPORTS),
            gdata,
            view,
        );
        a_box.append(&button);
    }
}
