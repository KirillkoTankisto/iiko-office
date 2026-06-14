use gtk4::{
    Align::Fill, Application, ApplicationWindow, Box, Orientation::Vertical, Stack, prelude::BoxExt,
};
use std::sync::Arc;

mod menu;
mod menubar;

use crate::gui::{
    GlobalData,
    main::{menu::MainMenu, menubar::MainMenuBar},
};

pub struct Main {
    root: Box
}

impl Main {
    pub fn new(gdata: Arc<GlobalData>, stack: &Stack, app: &Application, window: &ApplicationWindow) -> Self {
        let root = Box::builder()
        .orientation(Vertical)
        .spacing(8)
        .halign(Fill)
        .valign(Fill)
        .build();

        root.append(MainMenuBar::new(gdata.clone(), stack.clone(), app, window).present());
        root.append(MainMenu::new(gdata).present());

        Self {root}
    }

    pub fn present(&self) -> &Box {
        &self.root
    }
}
