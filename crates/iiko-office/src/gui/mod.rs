use gtk4::Application;
use gtk4::gio::SimpleAction;
use gtk4::{ApplicationWindow, prelude::*};

pub mod about;
pub mod common;
pub mod translation;

mod login;
mod main;

use crate::gui::common::global_data::GlobalData;
use crate::gui::login::LoginBox;
use crate::gui::main::Main;
use crate::gui::translation::CurrentLanguage::{self, EN, RU};
use std::env;

const APP_ID: &str = "org.fargo.iiko-office-libre";
const PRIMARY_KEY: &str = if cfg!(target_os = "macos") {
    "<Meta>"
} else {
    "<Ctrl>"
};

pub fn start_gui() {
    set_language();

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

// Should always be called before anything
fn set_language() {
    if env::var_os("LANG").is_none()
        && env::var_os("LC_ALL").is_none()
        && let Some(locale) = sys_locale::get_locale()
    {
        unsafe {
            env::set_var("LC_ALL", &locale);
            env::set_var("LANG", &locale);
        }
    }
}

fn get_language() -> CurrentLanguage {
    let locale_full = gtk4::default_language().to_string();

    let primary = locale_full
        .split(['-', '_', '.'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    match primary.as_str() {
        "ru" => RU,
        _ => EN,
    }
}
