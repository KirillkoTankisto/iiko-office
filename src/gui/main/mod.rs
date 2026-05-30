use gtk4::{Align::Fill, Application, Box, Orientation::Vertical, Stack, prelude::BoxExt};
use std::sync::{Arc, Mutex};

mod menu;
mod menubar;

use crate::gui::{
    GlobalData,
    main::{menu::create_menu, menubar::create_menubar},
};

pub fn create_main(gdata: Arc<Mutex<GlobalData>>, stack: Stack, app: &Application) -> Box {
    let main_box = Box::builder()
        .orientation(Vertical)
        .spacing(0)
        .halign(Fill)
        .valign(Fill)
        .build();

    let menubar = create_menubar(gdata, stack, app);
    let menu = create_menu();

    main_box.append(&menubar);
    main_box.append(&menu);

    return main_box;
}
