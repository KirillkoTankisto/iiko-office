use std::sync::Arc;

use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::prelude::*;

use crate::gui::common::global_data::GlobalData;
use crate::gui::login::LoginBox;
use crate::gui::main::Main;

const WINDOW_TITLE: &str = "iikoOffice";

pub fn on_activate(app: &Application, gdata: Arc<GlobalData>) {
    if let Some(window) = app.windows().first() {
        window.present(); // Focus on an existing window
        return;
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title(WINDOW_TITLE)
        .build();

    let stack = gtk4::Stack::builder()
        .hhomogeneous(false)
        .vhomogeneous(false)
        .build();
    window.set_child(Some(&stack));
    gdata.message_attach(&window);

    let main = Main::new(gdata.clone(), &stack, app, &window);
    let login = LoginBox::new(gdata.clone(), &stack, &main);

    stack.add_named(login.present(), Some("login"));
    stack.add_named(main.present(), Some("main"));

    stack.set_visible_child_name("login");
    window.present();
}
