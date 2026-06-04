use gtk4::{Box, Orientation::Horizontal, prelude::BoxExt};

use crate::gui::{
    main::menu::{buttons::create_buttons, view::create_view},
    translation::CurrentLanguage,
};

mod buttons;
pub mod tabs;
mod view;

pub fn create_menu(language: &CurrentLanguage) -> Box {
    let menu_box = Box::builder().spacing(0).orientation(Horizontal).build();

    let view = create_view();
    let buttons = create_buttons(&view, language);

    menu_box.append(&buttons);
    menu_box.append(&view);

    menu_box
}
