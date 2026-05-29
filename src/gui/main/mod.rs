use gtk4::{
    Align::Fill,
    Application, Box,
    Orientation::Vertical,
    PopoverMenuBar, Stack,
    gio::{Menu, prelude::ActionMapExt},
    prelude::BoxExt,
};

use crate::{api::logout::Logout, gui::GlobalData};
use std::sync::{Arc, Mutex};

pub fn create_main(gdata: Arc<Mutex<GlobalData>>, stack: Stack, app: &Application) -> Box {
    let main_box = Box::builder()
        .orientation(Vertical)
        .spacing(0)
        .halign(Fill)
        .valign(Fill)
        .build();

    let menubar = create_menubar(gdata, stack, app);
    main_box.append(&menubar);

    return main_box;
}

fn create_menubar(
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
