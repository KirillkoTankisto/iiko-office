use std::sync::Arc;

use gtk4::{Application, PopoverMenuBar, Stack, gio::Menu, gio::prelude::*};

use crate::{
    api::logout::Logout,
    gui::{
        GlobalData,
        translation::{
            Line::{FILE, LOGOUT},
            translate,
        },
    },
};

pub fn create_menubar(gdata: Arc<GlobalData>, stack: Stack, app: &Application) -> PopoverMenuBar {
    let menu = Menu::new();
    let file_menu = Menu::new();

    let logout_action = gtk4::gio::SimpleAction::new("logout", None);

    file_menu.append(
        Some(translate(gdata.language.clone(), LOGOUT)),
        Some("app.logout"),
    );
    menu.append_submenu(Some(translate(gdata.language.clone(), FILE)), &file_menu);

    logout_action.connect_activate(move |_, _| {
        logout_callback(gdata.clone(), stack.clone());
    });
    app.add_action(&logout_action);

    PopoverMenuBar::from_model(Some(&menu))
}

fn logout_callback(gdata: Arc<GlobalData>, stack: Stack) {
    let (sender, receiver) = async_channel::bounded(1);

    std::thread::spawn(move || {
        if let Ok(locked) = gdata.user_data.lock() {
            let logout = Logout::new(locked.address.clone(), locked.token.clone());

            if logout.run().is_ok() {
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
}
