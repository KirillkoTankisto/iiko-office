use gtk4::{Align::Fill, Application, Box, Orientation::Vertical, Stack, prelude::BoxExt};
use std::sync::Arc;

mod menu;
mod menubar;

use crate::gui::{
    GlobalData,
    main::{menu::create_menu, menubar::create_menubar},
};

pub fn create_main(gdata: Arc<GlobalData>, stack: Stack, app: &Application) -> Box {
    let main_box = Box::builder()
        .orientation(Vertical)
        .spacing(8)
        .halign(Fill)
        .valign(Fill)
        .build();

    let menubar = create_menubar(gdata.clone(), stack, app);
    let menu = create_menu(&gdata.language);

    main_box.append(&menubar);
    main_box.append(&menu);

    main_box
}
