use std::sync::Arc;

use gtk4::{Box, Notebook, Orientation::Vertical, prelude::BoxExt};

use crate::gui::{GlobalData, main::menu::buttons::cashshifts::create_cashshifts_button};

mod cashshifts;

pub fn create_buttons(gdata: Arc<GlobalData>, view: &Notebook) -> Box {
    let buttons_box = Box::builder()
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_bottom(8)
        .margin_top(8)
        .orientation(Vertical)
        .build();

    let cashshifts = create_cashshifts_button(gdata, view);

    buttons_box.append(&cashshifts);

    buttons_box
}
