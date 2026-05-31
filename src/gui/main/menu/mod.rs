use gtk4::{Box, Orientation::Horizontal, prelude::BoxExt};

use crate::gui::main::menu::{buttons::create_buttons, view::create_view};

mod buttons;
mod view;

pub fn create_menu() -> Box {
    let menu_box = Box::builder().spacing(8).orientation(Horizontal).build();

    let view = create_view();
    let buttons = create_buttons(&view);

    menu_box.append(&buttons);
    menu_box.append(&view);

    return menu_box;
}
