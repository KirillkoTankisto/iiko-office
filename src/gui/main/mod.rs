use gtk4::{Align::Fill, Application, ApplicationWindow, Box, Orientation::Vertical, Stack, prelude::BoxExt};
use std::sync::Arc;

mod menu;
mod menubar;

use crate::gui::{
    GlobalData,
    main::{menu::create_menu, menubar::create_menubar},
};

pub fn create_main(gdata: Arc<GlobalData>, stack: Stack, app: &Application, window: &ApplicationWindow) -> Box {
    let main_box = Box::builder()
        .orientation(Vertical)
        .spacing(8)
        .halign(Fill)
        .valign(Fill)
        .build();

    let menubar = create_menubar(gdata.clone(), stack, app, window);
    let menu = create_menu(gdata);

    main_box.append(&menubar);
    main_box.append(&menu);

    main_box
}
