use std::sync::Arc;

use gtk4::{
    Application, ApplicationWindow, PopoverMenuBar, Stack,
    gio::{Menu, prelude::*},
};

use gtk4::glib;

use crate::{
    api::logout::Logout,
    gui::{
        GlobalData,
        about::AboutPopup,
        common::utils::spawn_workflow,
        translation::{
            Line::{MENUBAR_ABOUT, MENUBAR_FILE, MENUBAR_LOGOUT},
            translate,
        },
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
        let menu = Menu::new();
        let file_menu = Menu::new();

        let logout_action = gtk4::gio::SimpleAction::new("logout", None);
        file_menu.append(
            Some(translate(gdata.language(), MENUBAR_LOGOUT)),
            Some("app.logout"),
        );

        let about_action = gtk4::gio::SimpleAction::new("about", None);
        file_menu.append(
            Some(translate(gdata.language(), MENUBAR_ABOUT)),
            Some("app.about"),
        );

        menu.append_submenu(Some(translate(gdata.language(), MENUBAR_FILE)), &file_menu);

        logout_action.connect_activate(glib::clone!(
            #[weak]
            stack,
            #[weak]
            gdata,
            move |_, _| {
                Self::logout_callback(gdata, stack);
            }
        ));

        about_action.connect_activate(glib::clone!(
            #[weak]
            window,
            move |_, _| {
                AboutPopup::new(&window, gdata.language()).present();
            }
        ));

        app.add_action(&logout_action);
        app.add_action(&about_action);

        Self {
            bar: PopoverMenuBar::from_model(Some(&menu)),
        }
    }

    pub fn present(&self) -> &PopoverMenuBar {
        &self.bar
    }

    fn logout_callback(gdata: Arc<GlobalData>, stack: Stack) {
        spawn_workflow(
            gdata,
            None,
            move |udata| Logout::new(&udata.address, &udata.token).run(),
            move |_| {
                stack.set_visible_child_name("login");
            },
        );
    }
}
