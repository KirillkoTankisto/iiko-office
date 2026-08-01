use crate::gui::common::global_data::GlobalData;
use gtk4::Application;
use std::sync::Arc;

pub fn on_shutdown(_app: &Application, gdata: Arc<GlobalData>) {
    if let Err(error) = gdata.write_config() {
        eprintln!("failed to write config on shutdown: {error}");
    }

    if let Some(session) = gdata.take_session()
        && let Err(error) = session.logout()
    {
        eprintln!("failed to log out on shutdown: {error}");
    }
}
