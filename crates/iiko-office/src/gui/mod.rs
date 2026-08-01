use gtk4::prelude::*;

pub mod about;
pub mod common;
pub mod translation;

mod application;
mod language;
mod login;
mod main;
mod on_activate;
mod on_shutdown;
mod on_startup;

use crate::gui::application::IikoOffice;
use crate::gui::common::global_data::GlobalData;
use crate::gui::language::set_language;
use crate::gui::on_activate::on_activate;
use crate::gui::on_shutdown::on_shutdown;
use crate::gui::on_startup::on_startup;

pub fn start_gui() {
    set_language();

    let app = IikoOffice::build();

    app.connect_activate(on_activate);
    app.connect_startup(on_startup);
    app.connect_shutdown(on_shutdown);

    app.run();
}
