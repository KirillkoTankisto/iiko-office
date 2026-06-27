use gtk4::{
    Align::Fill, Application, ApplicationWindow, Box, Orientation::Vertical, Stack, prelude::BoxExt,
};
use std::sync::Arc;

mod menu;
mod menubar;
mod statusbar;

use crate::gui::{
    GlobalData,
    main::{menu::MainMenu, menubar::MainMenuBar, statusbar::StatusBar},
};

use gtk4::glib;

#[derive(glib::Downgrade)]
pub struct Main {
    root: Box,
    statusbar: StatusBar
}

impl Main {
    pub fn new(
        gdata: Arc<GlobalData>,
        stack: &Stack,
        app: &Application,
        window: &ApplicationWindow,
    ) -> Self {
        let root = Box::builder()
            .orientation(Vertical)
            .spacing(8)
            .halign(Fill)
            .valign(Fill)
            .build();

        let statusbar = StatusBar::new(gdata.clone());

        root.append(MainMenuBar::new(gdata.clone(), stack.clone(), app, window).present());
        root.append(MainMenu::new(gdata).present());
        root.append(statusbar.present());

        Self { root, statusbar}
    }

    pub fn present(&self) -> &Box {
        &self.root
    }

    pub fn update_status(&self) {
        self.statusbar.update();
    }
}
