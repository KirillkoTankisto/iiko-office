use std::sync::Arc;

use gtk4::{Align, Button, Entry, Label, PasswordEntry, Stack, glib, prelude::*};
use iiko_api::{IikoConnection, consts::AsStr, utils::get_password_hash};

use crate::gui::{
    GlobalData,
    common::{
        anybox::AnyBox,
        dropdown::{AnyDropDown, DropDownItem},
        frame::frame,
        logo::logo_image,
        utils::spawn_task,
    },
    main::Main,
    translation::{
        CurrentLanguage,
        Line::{
            LOGIN, LOGIN_ADD_SERVER, LOGIN_ADDRESS, LOGIN_PASSWORD, LOGIN_REMOVE_SERVER,
            LOGIN_USERNAME,
        },
        translate,
    },
};

const FORM_WIDTH: i32 = 640;
const LOGO_SIZE: i32 = 128;

pub struct Credentials {
    pub address: String,
    pub username: String,
    pub password: String,
}

#[derive(glib::Downgrade)]
pub struct LoginBox {
    root: gtk4::Box,
    address: AddressBox,
    username: Entry,
    password: PasswordEntry,
    button: Button,
}

impl LoginBox {
    pub fn new(gdata: Arc<GlobalData>, stack: &Stack, main: &Main) -> Self {
        let logo = &logo_image(LOGO_SIZE);
        let title = &Label::builder()
            .label("iikoOffice")
            .margin_bottom(16)
            .css_classes(["title-2"])
            .build();

        let address = AddressBox::new(gdata.clone());
        let username = Entry::builder().hexpand(true).halign(Align::Fill).build();
        let password = PasswordEntry::builder()
            .hexpand(true)
            .halign(Align::Fill)
            .build();

        let button = Button::builder()
            .label(translate(gdata.language(), LOGIN))
            .margin_top(24)
            .hexpand(true)
            .halign(Align::Fill)
            .build();

        password.connect_activate(glib::clone!(
            #[weak]
            button,
            move |_| button.emit_clicked()
        ));

        let root = AnyBox::vertical()
            .align(Align::Center)
            .margin(16)
            .width_request(FORM_WIDTH)
            .add_widgets([
                logo.upcast_ref(),
                title.upcast_ref(),
                frame(gdata.language(), LOGIN_ADDRESS, &address.root).upcast_ref(),
                frame(gdata.language(), LOGIN_USERNAME, &username).upcast_ref(),
                frame(gdata.language(), LOGIN_PASSWORD, &password).upcast_ref(),
                button.upcast_ref(),
            ])
            .consume();

        let login_box = Self {
            root,
            address,
            username,
            password,
            button,
        };

        login_box.button.connect_clicked(glib::clone!(
            #[strong]
            gdata,
            #[weak]
            main,
            #[weak]
            stack,
            #[weak]
            login_box,
            move |button| {
                login_callback(gdata.clone(), button, login_box, stack, main);
            }
        ));

        login_box
    }

    pub fn present(&self) -> &gtk4::Box {
        &self.root
    }

    pub fn get_credentials(&self) -> Credentials {
        let address = self.address.url();
        let username = self.username.text().to_string();
        let password = self.password.text().to_string();

        Credentials {
            address,
            username,
            password,
        }
    }

    pub fn clear_password(&self) {
        self.password.delete_text(0, -1);
    }

    pub fn add_server(&self, address: &str) {
        self.address.add_server(address.to_string());
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Scheme {
    #[default]
    Https,
    Http,
}

impl AsStr for Scheme {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Https => "https://",
            Self::Http => "http://",
        }
    }
}

impl DropDownItem for Scheme {
    fn label(&self, _: CurrentLanguage) -> String {
        self.as_str().to_string()
    }
}

#[derive(glib::Downgrade)]
pub struct AddressBox {
    root: gtk4::Box,
    server_dropdown: AnyDropDown<String>,
    scheme_dropdown: AnyDropDown<Scheme>,
    entry: Entry,
    new_server_row: gtk4::Box,
}

impl AddressBox {
    fn new(gdata: Arc<GlobalData>) -> Self {
        let language = gdata.language();

        let server_dropdown =
            AnyDropDown::with_sentinel(language, -1, gdata.servers(), LOGIN_ADD_SERVER);
        server_dropdown.present().set_hexpand(true);
        server_dropdown.present().set_halign(Align::Fill);

        let delete_button = Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text(translate(language, LOGIN_REMOVE_SERVER))
            .build();

        let server_row = AnyBox::horizontal().add_widgets([
            server_dropdown.present().upcast_ref(),
            delete_button.upcast_ref(),
        ]);

        let scheme_dropdown = AnyDropDown::new(language, 90, vec![Scheme::Https, Scheme::Http]);
        let entry = Entry::builder().hexpand(true).halign(Align::Fill).build();

        let new_server_row = AnyBox::horizontal()
            .add_widgets([scheme_dropdown.present().upcast_ref(), entry.upcast_ref()]);

        let root = AnyBox::vertical().add_widgets([
            server_row.root().upcast_ref(),
            new_server_row.root().upcast_ref(),
        ]).consume();

        Self::sync(
            server_dropdown.is_sentinel_selected(),
            &new_server_row,
            &delete_button,
        );

        server_dropdown.connect_selected(glib::clone!(
            #[weak]
            new_server_row,
            #[weak]
            delete_button,
            move |server: Option<String>| {
                Self::sync(server.is_none(), &new_server_row, &delete_button);
            }
        ));

        delete_button.connect_clicked(glib::clone!(
            #[strong]
            gdata,
            #[weak]
            server_dropdown,
            #[weak]
            new_server_row,
            #[weak]
            delete_button,
            move |_| {
                let Some(address) = server_dropdown.remove_selected() else {
                    return;
                };

                gdata.remove_server(&address);
                if let Err(e) = gdata.write_config() {
                    gdata.message_send(e);
                }

                Self::sync(
                    server_dropdown.is_sentinel_selected(),
                    &new_server_row,
                    &delete_button,
                );
            }
        ));

        Self {
            root,
            server_dropdown,
            scheme_dropdown,
            entry,
            new_server_row: new_server_row.consume(),
        }
    }

    fn sync(adding_new: bool, new_server_row: &AnyBox, delete_button: &Button) {
        new_server_row.set_visible(adding_new);
        delete_button.set_sensitive(!adding_new);
    }

    fn add_server(&self, address: String) {
        if !self.server_dropdown.contains(&address) {
            self.server_dropdown.push(address);
        }
    }

    fn url(&self) -> String {
        match self.server_dropdown.selected() {
            Some(address) => address,
            None => format!(
                "{}{}",
                self.scheme_dropdown.selected().unwrap_or_default().as_str(),
                self.entry.text()
            ),
        }
    }
}

fn login_callback(
    gdata: Arc<GlobalData>,
    button: &Button,
    login_box: LoginBox,
    stack: Stack,
    main: Main,
) {
    let Credentials {
        address,
        username,
        password,
    } = login_box.get_credentials();

    if address.is_empty() || username.is_empty() || password.is_empty() {
        return;
    }

    spawn_task(
        gdata.clone(),
        Some(button),
        move || {
            let password_hashed = get_password_hash(&password);

            let session = IikoConnection::new(&address)?.auth(&username, &password_hashed)?;
            Ok((address, session))
        },
        move |(address, session)| {
            gdata.set_session(session);
            gdata.add_server(&address);

            login_box.clear_password();
            login_box.add_server(&address);
            main.update_status();
            stack.set_visible_child_name("main");
        },
    );
}
