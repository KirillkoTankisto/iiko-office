use gtk4::Application;
use gtk4::gio::SimpleAction;
use gtk4::{ApplicationWindow, prelude::*};

pub mod about;
pub mod common;
pub mod translation;

mod login;
mod main;

use crate::cfg::OfficeConfig;
use crate::gui::login::LoginBox;
use crate::gui::main::Main;
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

pub struct Address(Option<String>);
pub struct User(Option<String>);
pub struct Password(Option<String>);
pub struct Token(Option<String>);

pub struct GlobalData {
    user_data: Mutex<UserData>,
    language: CurrentLanguage,
    config: Mutex<OfficeConfig>,
}

impl GlobalData {
    pub fn new() -> Arc<GlobalData> {
        Arc::new(GlobalData {
            user_data: Mutex::new(UserData {
                address: String::default(),
                user: String::default(),
                password: String::default(),
                token: String::default(),
            }),
            language: get_language(),
            config: Mutex::new(OfficeConfig::load_config()),
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

    pub fn paste_credentials(
        &self,
        address: Address,
        user: User,
        password: Password,
        token: Token,
    ) {
        if let Ok(mut udata) = self.user_data.lock() {
            if let Some(address) = address.0 {
                udata.address = address
            };
            if let Some(user) = user.0 {
                udata.user = user
            };
            if let Some(password) = password.0 {
                udata.password = password
            };
            if let Some(token) = token.0 {
                udata.token = token
            };
        }
    }

    pub fn servers(&self) -> Vec<String> {
        self.config
            .lock()
            .map(|config| config.servers().to_vec())
            .unwrap_or_default()
    }

    pub fn add_server(&self, address: String) {
        if let Ok(mut config) = self.config.lock() {
            config.add_server(address);
        }
    }

    pub fn remove_server(&self, address: &str) {
        if let Ok(mut config) = self.config.lock() {
            config.remove_server(address);
        }
    }

    pub fn write_config(&self) {
        if let Ok(config) = self.config.lock() {
            config.write_config();
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

    let stack = gtk4::Stack::builder()
        .hhomogeneous(false)
        .vhomogeneous(false)
        .build();
    window.set_child(Some(&stack));

    let gdata = GlobalData::new();

    let main = Main::new(gdata.clone(), &stack, app, &window);
    let login = LoginBox::new(gdata.clone(), &stack, &main);

    stack.add_named(login.present(), Some("login"));
    stack.add_named(main.present(), Some("main"));

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
