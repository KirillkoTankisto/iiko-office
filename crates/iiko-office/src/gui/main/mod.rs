use gtk4::{Align::Fill, Box, Orientation::Vertical, prelude::BoxExt};
use std::sync::Arc;

mod menu;
mod statusbar;

use crate::gui::{
    GlobalData,
    main::{menu::MainMenu, statusbar::StatusBar},
};

use gtk4::glib;

#[derive(glib::Downgrade)]
pub struct Main {
    root: Box,
    statusbar: StatusBar,
}

impl Main {
    pub fn new(gdata: Arc<GlobalData>) -> Self {
        let root = Box::builder()
            .orientation(Vertical)
            .spacing(8)
            .halign(Fill)
            .valign(Fill)
            .build();

        let statusbar = StatusBar::new(gdata.clone());

        root.append(MainMenu::new(gdata).present());
        root.append(statusbar.present());

        Self { root, statusbar }
    }

    pub fn present(&self) -> &Box {
        &self.root
    }

    pub fn update_status(&self) {
        self.statusbar.update();
    }
}
