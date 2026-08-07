use gtk4::glib;
use gtk4::prelude::*;

pub mod about;
pub mod common;
pub mod translation;

mod language;
mod login;
mod main;
mod on_activate;
mod on_shutdown;
mod on_startup;

use crate::gui::common::global_data::GlobalData;
use crate::gui::language::set_language;
use crate::gui::on_activate::on_activate;
use crate::gui::on_shutdown::on_shutdown;
use crate::gui::on_startup::on_startup;

const APP_ID: &str = "org.fargo.iiko-office-libre";

pub fn start_gui() {
    set_language();

    let app = gtk4::Application::builder().application_id(APP_ID).build();
    let gdata = GlobalData::new();

    app.connect_startup(on_startup);

    app.connect_activate(glib::clone!(
        #[weak]
        gdata,
        move |app| on_activate(app, gdata.clone())
    ));

    app.connect_shutdown(glib::clone!(
        #[weak]
        gdata,
        move |app| on_shutdown(app, gdata)
    ));

    app.run();
}
