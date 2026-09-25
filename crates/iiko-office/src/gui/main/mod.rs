use gtk4::{Align::Fill, Box, Widget, prelude::*};
use std::sync::Arc;

mod menu;
mod statusbar;

use crate::gui::{
    GlobalData,
    common::anybox::AnyBox,
    main::{menu::MainMenu, statusbar::StatusBar},
};

use gtk4::glib;

#[derive(glib::Downgrade)]
pub struct Main {
    root: Box,
    status_bar: StatusBar,
}

impl Main {
    pub fn new(gdata: Arc<GlobalData>) -> Self {
        let status_bar = StatusBar::new(gdata.clone());
        let main_menu = MainMenu::new(gdata);

        let root = AnyBox::vertical()
            .align(Fill)
            .add_widgets([
                main_menu.present().upcast_ref::<Widget>(),
                status_bar.present().upcast_ref::<Widget>(),
            ])
            .consume();

        Self { root, status_bar }
    }

    pub fn present(&self) -> &Box {
        &self.root
    }

    pub fn update_status(&self) {
        self.status_bar.update();
    }
}
