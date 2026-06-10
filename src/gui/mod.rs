use gtk4::Application;
use gtk4::gio::SimpleAction;
use gtk4::{ApplicationWindow, prelude::*};

pub mod about;
pub mod common;
pub mod translation;

mod login;
mod main;

use crate::gui::login::LoginBox;
use crate::gui::main::create_main;
use crate::gui::translation::CurrentLanguage::{self, EN, RU};
use std::sync::{Arc, Mutex};

const APP_ID: &str = "org.fargo.iiko-office-libre";
const PRIMARY_KEY: &str = if cfg!(target_os = "macos") {
    "<Meta>"
} else {
    "<Ctrl>"
};

#[derive(Clone)]
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

type GlobalDataRef = Arc<GlobalData>;

impl GlobalData {
    pub fn new() -> GlobalDataRef {
        Arc::new(GlobalData {
            user_data: Mutex::new(UserData {
                address: String::default(),
                user: String::default(),
                password: String::default(),
                token: String::default(),
            }),
            language: get_language(),
        })
    }

    pub fn language(&self) -> CurrentLanguage {
        self.language
    }

    pub fn get_credentials(&self) -> Option<UserData> {
        if let Ok(udata) = self.user_data.lock() {
            Some(udata.clone())
        } else {
            None
        }
    }
}

pub fn start_gui() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.connect_startup(startup);

    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("iikoOffice")
        .build();

    let stack = gtk4::Stack::new();
    window.set_child(Some(&stack));

    let gdata = GlobalData::new();

    let login = LoginBox::new(gdata.clone(), stack.clone());
    let main = create_main(gdata.clone(), stack.clone(), app, &window);
    stack.add_named(&login.root, Some("login"));
    stack.add_named(&main, Some("main"));

    stack.set_visible_child_name("login");
    window.present();
}

fn startup(app: &Application) {
    let action_quit = SimpleAction::new("quit", None);

    app.add_action(&action_quit);

    app.set_accels_for_action("app.quit", &[&format!("{PRIMARY_KEY}q")]);
    app.set_accels_for_action("window.close", &[&format!("{PRIMARY_KEY}w")]);

    let action_app = app.clone();
    action_quit.connect_activate(move |_, _| action_app.clone().quit());
}

fn get_language() -> CurrentLanguage {
    let language_str = gtk4::default_language().to_string();
    if language_str.starts_with("ru") {
        RU
    } else {
        EN
    }
}
