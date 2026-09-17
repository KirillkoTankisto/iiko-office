use std::sync::Arc;

use gtk4::{Box, prelude::*};

use crate::gui::{
    GlobalData,
    common::anybox::AnyBox,
    main::menu::{buttons::create_buttons, view::MainView},
};

mod buttons;
pub mod tabs;
mod view;

pub struct MainMenu {
    root: Box,
}

impl MainMenu {
    pub fn new(gdata: Arc<GlobalData>) -> Self {
        let view = MainView::new();
        let buttons = create_buttons(gdata, &view);

        let root = AnyBox::horizontal()
            .add_widgets([buttons.upcast_ref(), view.present().upcast_ref()])
            .consume();

        Self { root }
    }

    pub fn present(&self) -> &Box {
        &self.root
    }
}
