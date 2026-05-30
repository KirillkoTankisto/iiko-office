use gtk4::Application;
use gtk4::{ApplicationWindow, prelude::*};

pub mod common;
mod login;
mod main;

use crate::gui::login::create_login;
use crate::gui::main::create_main;
use std::sync::{Arc, Mutex};

const APP_ID: &str = "org.fargo.iiko-office";

pub struct GlobalData {
    address: String,
    user: String,
    password: String,
    token: String,
}

pub fn start_gui() -> () {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run();

    return;
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("iikoOffice")
        .build();

    let stack = gtk4::Stack::new();
    window.set_child(Some(&stack));

    let gdata = Arc::new(Mutex::new(GlobalData {
        address: String::default(),
        user: String::default(),
        password: String::default(),
        token: String::default(),
    }));

    let login = create_login(gdata.clone(), stack.clone());
    let main = create_main(gdata.clone(), stack.clone(), app);
    stack.add_named(&login, Some("login"));
    stack.add_named(&main, Some("main"));

    stack.set_visible_child_name("login");
    window.present();
}
