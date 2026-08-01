use std::sync::Arc;

use crate::gui::{GlobalData, common::utils::spawn_workflow};
use gtk4::{glib, prelude::BoxExt};

#[derive(glib::Downgrade)]
pub struct StatusBar {
    root: gtk4::Box,
    left: gtk4::Label,
    right: gtk4::Label,
    gdata: Arc<GlobalData>,
}

impl StatusBar {
    pub fn new(gdata: Arc<GlobalData>) -> Self {
        let root = gtk4::Box::builder()
            .spacing(8)
            .orientation(gtk4::Orientation::Horizontal)
            .halign(gtk4::Align::Fill)
            .hexpand(true)
            .margin_bottom(16)
            .margin_top(0)
            .margin_start(16)
            .margin_end(16)
            .build();

        let left = gtk4::Label::builder()
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();
        let right = gtk4::Label::builder().halign(gtk4::Align::End).build();

        root.append(&left);
        root.append(&right);

        Self {
            root,
            left,
            right,
            gdata,
        }
    }

    pub fn update(&self) {
        let sleft = self.left.clone();
        let sright = self.right.clone();

        spawn_workflow(
            self.gdata.clone(),
            None,
            move |session| {
                let user = session.user().to_string();
                session.version().map(|version| (user, version))
            },
            move |(user, version)| {
                let left = format!(
                    "{} {}, {} ({})",
                    version.version, version.edition, version.server_name, version.computer_name
                );
                let right = format!("{}, {}", user, version.server_state);
                sleft.set_label(&left);
                sright.set_label(&right);
            },
        );
    }

    pub fn present(&self) -> &gtk4::Box {
        &self.root
    }
}
