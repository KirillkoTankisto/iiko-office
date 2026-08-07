use gtk4::prelude::*;
use gtk4::{Application, gio::SimpleAction, glib};

const PRIMARY_KEY: &str = if cfg!(target_os = "macos") {
    "<Meta>"
} else {
    "<Ctrl>"
};

pub fn on_startup(app: &Application) {
    let action_quit = SimpleAction::new("quit", None);
    action_quit.connect_activate(glib::clone!(
        #[weak]
        app,
        move |_, _| app.quit()
    ));

    app.add_action(&action_quit);
    app.set_accels_for_action("app.quit", &[&format!("{PRIMARY_KEY}q")]);
    app.set_accels_for_action("window.close", &[&format!("{PRIMARY_KEY}w")]);
}
