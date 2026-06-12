use std::sync::Arc;

use gtk4::{Box, Notebook, Orientation::Vertical, prelude::BoxExt};

use crate::gui::{GlobalData, main::menu::buttons::{cashshifts::create_cashshifts_button, olap::create_olap_button}};

mod cashshifts;
mod olap;

pub fn create_buttons(gdata: Arc<GlobalData>, view: &Notebook) -> Box {
    let buttons_box = Box::builder()
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_bottom(8)
        .margin_top(8)
        .orientation(Vertical)
        .build();

    buttons_box.append(&create_cashshifts_button(gdata.clone(), view));
    buttons_box.append(&create_olap_button(gdata, view));

    buttons_box
}
