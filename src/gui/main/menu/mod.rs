use std::sync::Arc;

use gtk4::{Box, Orientation::Horizontal, prelude::BoxExt};

use crate::gui::{
    GlobalData, main::menu::{buttons::create_buttons, view::create_view}
};

mod buttons;
pub mod tabs;
mod view;

pub fn create_menu(gdata: Arc<GlobalData>) -> Box {
    let menu_box = Box::builder().spacing(0).orientation(Horizontal).build();

    let view = create_view();
    let buttons = create_buttons(gdata, &view);

    menu_box.append(&buttons);
    menu_box.append(&view);

    menu_box
}
