use std::sync::Arc;

use gtk4::{Box, Orientation::Horizontal, prelude::BoxExt};

use crate::gui::{
    GlobalData,
    main::menu::{buttons::create_buttons, view::MainView},
};

mod buttons;
pub mod tabs;
mod view;

pub struct MainMenu {
    root: Box
}

impl MainMenu {
    pub fn new(gdata: Arc<GlobalData>) -> Self {
        let root = Box::builder().spacing(0).orientation(Horizontal).build();

        let view = MainView::new();

        let buttons = create_buttons(gdata, &view);

        root.append(&buttons);
        root.append(view.present());

        Self {root}
    }

    pub fn present(&self) -> &Box {
        &self.root
    }
}
