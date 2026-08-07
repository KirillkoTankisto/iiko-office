use std::sync::Arc;

use gtk4::{
    Application, ApplicationWindow, Stack,
    gio::{Menu, MenuItem, SimpleAction},
    glib,
};

use gtk4::prelude::*;

use crate::gui::{
    GlobalData, about,
    common::utils::spawn_task,
    translation::{
        CurrentLanguage,
        Line::{
            self, MENUBAR_ABOUT, MENUBAR_COPY, MENUBAR_CUT, MENUBAR_EDIT, MENUBAR_FILE,
            MENUBAR_LOGOUT, MENUBAR_PASTE, MENUBAR_QUIT, MENUBAR_REDO, MENUBAR_SELECT_ALL,
            MENUBAR_UNDO,
        },
        translate,
    },
};

#[cfg(target_os = "macos")]
use crate::gui::translation::Line::MENUBAR_WINDOW;

pub struct GlobalMenuBar;

impl GlobalMenuBar {
    pub fn install(
        gdata: Arc<GlobalData>,
        stack: &Stack,
        app: &Application,
        window: &ApplicationWindow,
    ) -> SimpleAction {
        let language = gdata.language();
        let menubar = Menu::new();

        // File
        let file = Menu::new();

        let logout = add_action(
            app,
            "logout",
            glib::clone!(
                #[weak]
                stack,
                #[weak]
                gdata,
                move || Self::logout_callback(gdata, stack)
            ),
        );

        let sync = |stack: &Stack, action: &SimpleAction| {
            action.set_enabled(stack.visible_child_name().as_deref() == Some("main"));
        };

        sync(stack, &logout);
        stack.connect_visible_child_name_notify(glib::clone!(
            #[weak]
            logout,
            move |stack| sync(stack, &logout)
        ));

        file.append_item(&item(language, MENUBAR_LOGOUT, "app.logout", false));

        add_action(
            app,
            "about",
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

        // Hide About and Quit on macOS.
        // macOS already ships those actions
        let extras = Menu::new();
        extras.append_item(&item(language, MENUBAR_ABOUT, "app.about", true));
        extras.append_item(&item(language, MENUBAR_QUIT, "app.quit", true));
        file.append_section(None, &extras);

        menubar.append_submenu(Some(translate(language, MENUBAR_FILE)), &file);

        // Edit
        let edit = Menu::new();

        let history = Menu::new();
        history.append(Some(translate(language, MENUBAR_UNDO)), Some("text.undo"));
        history.append(Some(translate(language, MENUBAR_REDO)), Some("text.redo"));
        edit.append_section(None, &history);

        let clipboard = Menu::new();
        clipboard.append(
            Some(translate(language, MENUBAR_CUT)),
            Some("clipboard.cut"),
        );
        clipboard.append(
            Some(translate(language, MENUBAR_COPY)),
            Some("clipboard.copy"),
        );
        clipboard.append(
            Some(translate(language, MENUBAR_PASTE)),
            Some("clipboard.paste"),
        );
        clipboard.append(
            Some(translate(language, MENUBAR_SELECT_ALL)),
            Some("selection.select-all"),
        );
        edit.append_section(None, &clipboard);

        menubar.append_submenu(Some(translate(language, MENUBAR_EDIT)), &edit);

        // Window (only on macOS)
        #[cfg(target_os = "macos")]
        {
            let windows = MenuItem::new(Some(translate(language, MENUBAR_WINDOW)), None);
            windows.set_submenu(Some(&Menu::new()));
            windows.set_attribute_value("gtk-macos-special", Some(&"window-submenu".to_variant()));
            menubar.append_item(&windows);
        }

        app.set_menubar(Some(&menubar));

        window.set_show_menubar(true);

        logout
    }

    fn logout_callback(gdata: Arc<GlobalData>, stack: Stack) {
        stack.set_visible_child_name("login");
        let Some(session) = gdata.take_session() else {
            return;
        };
        spawn_task(gdata, None, move || Ok(session.logout()?), |_| {});
    }
}

fn add_action(app: &Application, name: &str, callback: impl Fn() + 'static) -> SimpleAction {
    let action = SimpleAction::new(name, None);
    action.connect_activate(move |_, _| callback());
    app.add_action(&action);
    action
}

fn item(language: CurrentLanguage, line: Line, action: &str, macos_hidden: bool) -> MenuItem {
    let entry = MenuItem::new(Some(translate(language, line)), Some(action));

    if macos_hidden {
        entry.set_attribute_value("hidden-when", Some(&"macos-menubar".to_variant()));
    }

    entry
}
