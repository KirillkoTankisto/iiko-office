use gtk4::Application;
use gtk4::prelude::*;

pub fn on_shutdown(app: &Application) {
    app.quit();
}
