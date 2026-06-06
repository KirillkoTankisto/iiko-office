use gtk4::Application;
use gtk4::{ApplicationWindow, prelude::*};

pub mod about;
pub mod common;
pub mod translation;

mod login;
mod main;

use crate::gui::login::create_login;
use crate::gui::main::create_main;
use crate::gui::translation::CurrentLanguage::{self, EN, RU};
use std::sync::{Arc, Mutex};

const APP_ID: &str = "org.fargo.iiko-office";

pub struct UserData {
    address: String,
    user: String,
    password: String,
    token: String,
}

pub struct GlobalData {
    user_data: Mutex<UserData>,
    language: CurrentLanguage,
}

pub fn start_gui() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("iikoOffice")
        .build();

    let stack = gtk4::Stack::new();
    window.set_child(Some(&stack));

    let gdata = Arc::new(GlobalData {
        user_data: Mutex::new(UserData {
            address: String::default(),
            user: String::default(),
            password: String::default(),
            token: String::default(),
        }),
        language: get_language(),
    });

    let login = create_login(gdata.clone(), stack.clone());
    let main = create_main(gdata.clone(), stack.clone(), app, &window);
    stack.add_named(&login, Some("login"));
    stack.add_named(&main, Some("main"));

    stack.set_visible_child_name("login");
    window.present();
}

fn get_language() -> CurrentLanguage {
    let language_str = gtk4::default_language().to_string();
    if language_str.starts_with("ru") {
        RU
    } else {
        EN
    }
}
