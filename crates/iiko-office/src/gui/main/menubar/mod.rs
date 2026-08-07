use std::sync::Arc;

use gtk4::{
    Application, ApplicationWindow, PopoverMenuBar, Stack,
    gio::{Menu, SimpleAction, prelude::*},
    glib,
};

use crate::gui::{
    GlobalData, about,
    common::utils::spawn_task,
    translation::{
        CurrentLanguage,
        Line::{self, MENUBAR_ABOUT, MENUBAR_FILE, MENUBAR_LOGOUT},
        translate,
    },
};

pub struct MainMenuBar {
    bar: PopoverMenuBar,
}

impl MainMenuBar {
    pub fn new(
        gdata: Arc<GlobalData>,
        stack: Stack,
        app: &Application,
        window: &ApplicationWindow,
    ) -> Self {
        let language = gdata.language();
        let menu = Menu::new();
        let file_menu = Menu::new();

        add_item(
            app,
            &file_menu,
            language,
            "logout",
            MENUBAR_LOGOUT,
            glib::clone!(
                #[weak]
                stack,
                #[weak]
                gdata,
                move || Self::logout_callback(gdata, stack)
            ),
        );

        add_item(
            app,
            &file_menu,
            language,
            "about",
            MENUBAR_ABOUT,
            glib::clone!(
                #[weak]
                window,
                #[weak]
                gdata,
                move || {
                    glib::spawn_future_local(async move {
                        about::show_about(&window, gdata.language()).await
                    });
                }
            ),
        );

        menu.append_submenu(Some(translate(language, MENUBAR_FILE)), &file_menu);

        Self {
            bar: PopoverMenuBar::from_model(Some(&menu)),
        }
    }

    pub fn present(&self) -> &PopoverMenuBar {
        &self.bar
    }

    fn logout_callback(gdata: Arc<GlobalData>, stack: Stack) {
        stack.set_visible_child_name("login");
        let Some(session) = gdata.take_session() else {
            return;
        };
        spawn_task(gdata, None, move || Ok(session.logout()?), |_| {});
    }
}

/// Registers `app.<name>` and adds the matching entry to `menu`.
fn add_item(
    app: &Application,
    menu: &Menu,
    language: CurrentLanguage,
    name: &str,
    line: Line,
    callback: impl Fn() + 'static,
) {
    let action = SimpleAction::new(name, None);
    action.connect_activate(move |_, _| callback());

    menu.append(
        Some(translate(language, line)),
        Some(&format!("app.{name}")),
    );
    app.add_action(&action);
}
