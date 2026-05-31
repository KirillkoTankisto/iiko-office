use std::sync::{Arc, Mutex};

use gtk4::{Application, PopoverMenuBar, Stack, gio::Menu, gio::prelude::*};

use crate::{api::logout::Logout, gui::GlobalData};

pub fn create_menubar(
    gdata: Arc<Mutex<GlobalData>>,
    stack: Stack,
    app: &Application,
) -> PopoverMenuBar {
    let menu = Menu::new();
    let file_menu = Menu::new();

    let logout_action = gtk4::gio::SimpleAction::new("logout", None);
    logout_action.connect_activate(move |_, _| {
        logout_callback(gdata.clone(), stack.clone());
    });

    file_menu.append(Some("Quit"), Some("app.logout"));
    menu.append_submenu(Some("File"), &file_menu);

    app.add_action(&logout_action);

    return PopoverMenuBar::from_model(Some(&menu));
}

fn logout_callback(gdata: Arc<Mutex<GlobalData>>, stack: Stack) {
    let (sender, receiver) = async_channel::bounded(1);

    std::thread::spawn(move || {
        if let Ok(locked) = gdata.lock() {
            let logout = Logout::new(locked.address.clone(), locked.token.clone());
            if let Ok(_) = logout.run() {
                println!("Logout success");
            } else {
                println!("Logout failure");
            }
        }

        let _ = sender.send_blocking(());
    });

    gtk4::glib::spawn_future_local(async move {
        let _ = receiver.recv().await;
        stack.set_visible_child_name("login");
    });

    return;
}
