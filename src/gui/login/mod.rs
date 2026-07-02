use std::sync::Arc;

use gtk4::Button;
use gtk4::DropDown;
use gtk4::StringList;
use gtk4::StringObject;
use gtk4::prelude::*;

use gtk4::Align;
use gtk4::Entry;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::PasswordEntry;
use gtk4::Stack;

use gtk4::glib;

use crate::api::auth::*;
use crate::gui::GlobalData;
use crate::gui::common::logo::get_logo_image;
use crate::gui::common::utils::spawn_workflow;
use crate::gui::main::Main;
use crate::gui::translation::Line::LOGIN;
use crate::gui::translation::Line::LOGIN_ADD_SERVER;
use crate::gui::translation::Line::LOGIN_ADDRESS;
use crate::gui::translation::Line::LOGIN_PASSWORD;
use crate::gui::translation::Line::LOGIN_REMOVE_SERVER;
use crate::gui::translation::Line::LOGIN_USERNAME;
use crate::gui::translation::translate;

const FORM_WIDTH: i32 = 640;

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
    error: Label,
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

        root.append(&get_logo_image());
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

        let error = Label::builder()
            .label("Some Error")
            .wrap(true)
            .max_width_chars(32)
            .wrap_mode(gtk4::pango::WrapMode::Char)
            .halign(Align::Center)
            .margin_top(8)
            .visible(false)
            .build();

        root.append(&error);

        let login_box = Self {
            root,
            address,
            username,
            password,
            button,
            error,
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
                let sentinel = server_list.n_items().saturating_sub(1);
                let on_sentinel = dropdown.selected() == sentinel;
                new_server_row.set_visible(on_sentinel);
                delete_button.set_sensitive(!on_sentinel);
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
                let sentinel = server_list.n_items().saturating_sub(1);

                if selected >= sentinel {
                    return;
                }

                if let Some(address) = server_list.string(selected) {
                    gdata.remove_server(address.as_str());
                    gdata.write_config();
                }

                server_list.remove(selected);

                let on_sentinel =
                    server_dropdown.selected() == server_list.n_items().saturating_sub(1);
                new_server_row.set_visible(on_sentinel);
                delete_button.set_sensitive(!on_sentinel);
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

    fn add_server(&self, address: &str) {
        let sentinel = self.servers.n_items().saturating_sub(1);

        let exists = (0..sentinel)
            .filter_map(|i| self.servers.string(i))
            .any(|s| s.as_str() == address);
        if exists {
            return;
        }

        self.servers.splice(sentinel, 0, &[address]);
    }

    fn is_add_new_selected(&self) -> bool {
        self.server_dropdown.selected() == self.servers.n_items().saturating_sub(1)
    }

    fn url(&self) -> String {
        if self.is_add_new_selected() {
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

    spawn_workflow(
        gdata.clone(),
        Some(button),
        move |_| {
            let password_hashed = get_password_hash(&password);

            let auth = Auth::new(&address, &username, &password_hashed);
            let result = auth.run();
            if let Ok(token) = &result {
                gdata.paste_credentials(
                    Some(&address),
                    Some(&username),
                    Some(&password),
                    Some(token),
                );
            }
            result.map(|_| address)
        },
        move |address| {
            login_box.add_server(&address);
            main.update_status();
            stack.set_visible_child_name("main");
        },
    );
}
