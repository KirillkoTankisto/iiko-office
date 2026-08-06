use std::sync::Arc;

use gtk4::{
    Align, Button, DropDown, Entry, Label, Orientation, PasswordEntry, Stack, StringList,
    StringObject, glib, prelude::*,
};
use iiko_api::{IikoConnection, utils::get_password_hash};

use crate::gui::{
    GlobalData,
    common::{logo::logo_image, utils::spawn_task},
    main::Main,
    translation::{
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

#[derive(Clone, glib::Downgrade)]
pub struct LoginBox {
    root: gtk4::Box,
    address: AddressBox,
    username: Entry,
    password: PasswordEntry,
    button: Button,
}

impl LoginBox {
    pub fn new(gdata: Arc<GlobalData>, stack: &Stack, main: &Main) -> Self {
        let root = gtk4::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .margin_start(64)
            .margin_end(64)
            .margin_bottom(16)
            .margin_top(16)
            .halign(Align::Center)
            .valign(Align::Center)
            .width_request(FORM_WIDTH)
            .build();

        root.append(&logo_image(LOGO_SIZE));
        root.append(
            &Label::builder()
                .label("iikoOffice")
                .margin_bottom(16)
                .css_classes(["title-2"])
                .build(),
        );

        let label = |line| {
            Label::builder()
                .label(translate(gdata.language(), line))
                .halign(Align::Start)
                .margin_top(8)
                .build()
        };

        let address = AddressBox::new(gdata.clone());
        let username = Entry::builder().hexpand(true).halign(Align::Fill).build();
        let password = PasswordEntry::builder()
            .hexpand(true)
            .halign(Align::Fill)
            .build();

        root.append(&label(LOGIN_ADDRESS));
        root.append(&address.root);
        root.append(&label(LOGIN_USERNAME));
        root.append(&username);
        root.append(&label(LOGIN_PASSWORD));
        root.append(&password);

        let button = Button::builder()
            .label(translate(gdata.language(), LOGIN))
            .margin_top(24)
            .hexpand(true)
            .halign(Align::Fill)
            .build();

        root.append(&button);

        password.connect_activate(glib::clone!(
            #[weak]
            button,
            move |_| button.emit_clicked()
        ));

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
        self.address.add_server(address);
    }
}

#[derive(Clone, glib::Downgrade)]
pub struct AddressBox {
    root: gtk4::Box,
    servers: StringList,
    server_dropdown: DropDown,
    scheme_dropdown: DropDown,
    entry: Entry,
    new_server_row: gtk4::Box,
}

impl AddressBox {
    fn new(gdata: Arc<GlobalData>) -> Self {
        let servers = gdata.servers();
        let language = gdata.language();

        let root = gtk4::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();

        let server_list = StringList::new(&[]);
        for server in &servers {
            server_list.append(server);
        }
        server_list.append(translate(language, LOGIN_ADD_SERVER));

        let server_row = gtk4::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let server_dropdown = DropDown::builder()
            .model(&server_list)
            .hexpand(true)
            .halign(Align::Fill)
            .build();

        let delete_button = Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text(translate(gdata.language(), LOGIN_REMOVE_SERVER))
            .build();

        delete_button.set_sensitive(!servers.is_empty());

        server_row.append(&server_dropdown);
        server_row.append(&delete_button);

        let new_server_row = gtk4::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        let scheme_dropdown = DropDown::from_strings(&["https://", "http://"]);
        scheme_dropdown.set_size_request(90, -1);
        let entry = Entry::builder().hexpand(true).halign(Align::Fill).build();
        new_server_row.append(&scheme_dropdown);
        new_server_row.append(&entry);

        root.append(&server_row);
        root.append(&new_server_row);

        new_server_row.set_visible(servers.is_empty());

        server_dropdown.connect_selected_notify(glib::clone!(
            #[weak]
            new_server_row,
            #[weak]
            server_list,
            #[weak]
            delete_button,
            move |dropdown| {
                Self::sync_add_server_row(dropdown, &server_list, &new_server_row, &delete_button);
            }
        ));

        delete_button.connect_clicked(glib::clone!(
            #[strong]
            gdata,
            #[weak]
            server_dropdown,
            #[weak]
            server_list,
            #[weak]
            new_server_row,
            #[weak]
            delete_button,
            move |_| {
                let selected = server_dropdown.selected();

                if selected >= Self::add_server_index(&server_list) {
                    return;
                }

                if let Some(address) = server_list.string(selected) {
                    gdata.remove_server(address.as_str());
                    if let Err(e) = gdata.write_config() {
                        gdata.message_send(e);
                    }
                }

                server_list.remove(selected);

                Self::sync_add_server_row(
                    &server_dropdown,
                    &server_list,
                    &new_server_row,
                    &delete_button,
                );
            }
        ));

        Self {
            root,
            servers: server_list,
            server_dropdown,
            scheme_dropdown,
            entry,
            new_server_row,
        }
    }

    fn add_server_index(list: &StringList) -> u32 {
        list.n_items().saturating_sub(1)
    }

    fn sync_add_server_row(
        dropdown: &DropDown,
        list: &StringList,
        new_server_row: &gtk4::Box,
        delete_button: &Button,
    ) {
        let adding_new = dropdown.selected() == Self::add_server_index(list);
        new_server_row.set_visible(adding_new);
        delete_button.set_sensitive(!adding_new);
    }

    fn add_server(&self, address: &str) {
        let index = Self::add_server_index(&self.servers);

        let exists = (0..index)
            .filter_map(|i| self.servers.string(i))
            .any(|s| s.as_str() == address);
        if exists {
            return;
        }

        self.servers.splice(index, 0, &[address]);
    }

    fn is_add_server_selected(&self) -> bool {
        self.server_dropdown.selected() == Self::add_server_index(&self.servers)
    }

    fn url(&self) -> String {
        if self.is_add_server_selected() {
            let scheme = self
                .scheme_dropdown
                .selected_item()
                .and_downcast::<StringObject>()
                .map(|s| s.string().to_string())
                .unwrap_or("https://".into());

            format!("{scheme}{}", self.entry.text())
        } else {
            self.servers
                .string(self.server_dropdown.selected())
                .map(|s| s.to_string())
                .unwrap_or_default()
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
