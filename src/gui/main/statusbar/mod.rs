use std::sync::Arc;

use crate::{
    api::get_version::GetVersion,
    gui::{GlobalData, common::utils::spawn_workflow},
};
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
            move |udata| {
                GetVersion::new(&udata.address)
                    .run()
                    .map(|result| (udata.user.clone(), result))
            },
            move |version| {
                let left = format!(
                    "{} {}, {} ({})",
                    version.1.version,
                    version.1.edition,
                    version.1.serverName,
                    version.1.computerName
                );
                let right = format!("{}, {}", version.0, version.1.serverState);
                sleft.set_label(&left);
                sright.set_label(&right);
            },
        );
    }

    pub fn present(&self) -> &gtk4::Box {
        &self.root
    }
}
