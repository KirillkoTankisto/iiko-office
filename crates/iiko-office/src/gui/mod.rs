use gtk4::prelude::*;

pub mod about;
pub mod common;
pub mod translation;

mod app;
mod global_menu;
mod language;
mod login;
mod main;
mod on_activate;
mod on_shutdown;
mod on_startup;

use crate::gui::app::create_app;
use crate::gui::common::global_data::GlobalData;
use crate::gui::common::utils::with;
use crate::gui::language::set_language;
use crate::gui::on_activate::on_activate;
use crate::gui::on_shutdown::on_shutdown;
use crate::gui::on_startup::on_startup;

pub fn start_gui() {
    set_language();

    let app = create_app();
    let gdata = GlobalData::new();

    app.connect_startup(on_startup);
    app.connect_activate(with(&gdata, on_activate));
    app.connect_shutdown(with(&gdata, on_shutdown));

    app.run();
}
