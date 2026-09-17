use std::sync::Arc;

use crate::gui::{
    GlobalData,
    common::{anybox::AnyBox, utils::spawn_workflow},
};
use gtk4::{glib, prelude::*};

#[derive(glib::Downgrade)]
pub struct StatusBar {
    root: gtk4::Box,
    left: gtk4::Label,
    right: gtk4::Label,
    gdata: Arc<GlobalData>,
}

impl StatusBar {
    pub fn new(gdata: Arc<GlobalData>) -> Self {
        let left = gtk4::Label::builder()
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();
        let right = gtk4::Label::builder().halign(gtk4::Align::End).build();

        let root = AnyBox::horizontal()
            .align(gtk4::Align::Fill)
            .margin(16)
            .add_widgets([left.upcast_ref(), right.upcast_ref()])
            .consume();

        Self {
            root,
            left,
            right,
            gdata,
        }
    }

    pub fn update(&self) {
        let left = self.left.clone();
        let right = self.right.clone();

        spawn_workflow(
            self.gdata.clone(),
            None,
            move |session| {
                let user = session.user().to_string();
                session.version().map(|version| (user, version))
            },
            move |(user, version)| {
                let left_text = format!(
                    "{} {}, {} ({})",
                    version.version, version.edition, version.server_name, version.computer_name
                );
                let right_text = format!("{}, {}", user, version.server_state);
                left.set_label(&left_text);
                right.set_label(&right_text);
            },
        );
    }

    pub fn present(&self) -> &gtk4::Box {
        &self.root
    }
}
