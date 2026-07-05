use gtk4::Application;
use gtk4::gio::SimpleAction;
use gtk4::{ApplicationWindow, prelude::*};
use iiko_api::IikoSession;

pub mod about;
pub mod common;
pub mod translation;

mod login;
mod main;

use crate::cfg::OfficeConfig;
use crate::error::AppError;
use crate::gui::common::message_bus::MessageBus;
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

pub struct GlobalData {
    session: Mutex<Option<Arc<IikoSession>>>,
    language: CurrentLanguage,
    config: Mutex<OfficeConfig>,
    message_bus: MessageBus,
}

impl GlobalData {
    pub fn new() -> Arc<GlobalData> {
        Arc::new(GlobalData {
            session: Mutex::new(None),
            language: get_language(),
            config: Mutex::new(OfficeConfig::load_config()),
            message_bus: MessageBus::new(),
        })
    }

    pub fn language(&self) -> CurrentLanguage {
        self.language
    }

    pub fn session(&self) -> Result<Arc<IikoSession>, AppError> {
        self.session
            .lock()
            .map_err(|_| AppError::Internal)?
            .clone()
            .ok_or(AppError::NotLoggedIn)
    }

    pub fn set_session(&self, session: IikoSession) {
        if let Ok(mut locked) = self.session.lock() {
            *locked = Some(Arc::new(session))
        }
    }

    pub fn take_session(&self) -> Option<Arc<IikoSession>> {
        self.session.lock().ok()?.take()
    }

    pub fn servers(&self) -> Vec<String> {
        self.config
            .lock()
            .map(|config| config.servers().to_vec())
            .unwrap_or_default()
    }

    pub fn add_server(&self, address: &str) {
        if let Ok(mut config) = self.config.lock() {
            config.add_server(address);
        }
    }

    pub fn remove_server(&self, address: &str) {
        if let Ok(mut config) = self.config.lock() {
            config.remove_server(address);
        }
    }

    pub fn write_config(&self) -> Result<(), AppError> {
        let config = self.config.lock().map_err(|_| AppError::Internal)?;
        Ok(config.write_config()?)
    }

    pub fn message_send(&self, error: AppError) {
        self.message_bus.emit(error);
    }

    pub fn message_attach(&self, window: &ApplicationWindow) {
        self.message_bus.attach(window, self.language);
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
    gdata.message_attach(&window);

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
