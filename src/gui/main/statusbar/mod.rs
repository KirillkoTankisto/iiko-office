use std::sync::Arc;

use crate::{api::get_version::GetVersion, gui::GlobalData};
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

        let left = gtk4::Label::builder().halign(gtk4::Align::Start).hexpand(true).build();
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
        if let Some(udata) = self.gdata.get_credentials() {
            let (sender, receiver) = async_channel::bounded(1);

            std::thread::spawn(move || {
                _ = sender.send_blocking(GetVersion::new(udata.address).run());
            });

            let sleft = &self.left;
            let sright = &self.right;

            glib::spawn_future_local(glib::clone!(
                #[weak]
                sleft,
                #[weak]
                sright,
                async move {
                    if let Ok(received) = receiver.recv().await {
                        match received {
                            Ok(version) => {
                                let left = format!(
                                    "{} {}, {} ({})",
                                    version.version,
                                    version.edition,
                                    version.serverName,
                                    version.computerName
                                );
                                let right = format!("{}, {}", udata.user, version.serverState);
                                sleft.set_label(&left);
                                sright.set_label(&right);
                            }
                            Err(e) => eprintln!("{e}"),
                        }
                    }
                }
            ));
        }
    }

    pub fn present(&self) -> &gtk4::Box {
        &self.root
    }
}
