use std::sync::Arc;

use gtk4::Button;
use gtk4::DropDown;
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
use crate::gui::translation::Line::ADDRESS;
use crate::gui::translation::Line::LOGIN;
use crate::gui::translation::Line::PASSWORD;
use crate::gui::translation::Line::USERNAME;
use crate::gui::translation::translate;

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
    pub fn new(gdata: Arc<GlobalData>, stack: &Stack) -> Self {
        let root = gtk4::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_start(64)
            .margin_end(64)
            .margin_bottom(16)
            .margin_top(16)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        let label = |line| Label::new(Some(translate(gdata.language(), line)));

        let address = AddressBox::new();
        let username = Entry::builder().width_chars(32).build();
        let password = PasswordEntry::builder().width_chars(32).build();

        root.append(&label(ADDRESS));
        root.append(&address.root);
        root.append(&label(USERNAME));
        root.append(&username);
        root.append(&label(PASSWORD));
        root.append(&password);

        let button = Button::builder()
            .label(translate(gdata.language(), LOGIN))
            .margin_top(24)
            .width_request(360)
            .halign(Align::Fill)
            .build();

        root.append(&button);

        let error = Label::builder()
            .label("Some Error")
            .wrap(true)
            .max_width_chars(32)
            .wrap_mode(gtk4::pango::WrapMode::Char)
            .halign(Align::Center)
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
            stack,
            #[weak]
            login_box,
            move |_| {
                login_callback(gdata.clone(), stack, login_box);
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

    pub fn set_sensitive(&self, value: bool) {
        self.button.set_sensitive(value);
    }
}

#[derive(Clone, glib::Downgrade)]
pub struct AddressBox {
    root: gtk4::Box,
    dropdown: DropDown,
    entry: Entry,
}

impl AddressBox {
    fn new() -> Self {
        let root = gtk4::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        let dropdown = DropDown::from_strings(&["https://", "http://"]);
        dropdown.set_size_request(90, -1);
        let entry = Entry::builder().width_chars(32).build();

        root.append(&dropdown);
        root.append(&entry);

        Self {
            root,
            dropdown,
            entry,
        }
    }

    fn url(&self) -> String {
        let scheme = self
            .dropdown
            .selected_item()
            .and_downcast::<gtk4::StringObject>()
            .map(|s| s.string().to_string())
            .unwrap_or("https://".into());

        format!("{scheme}{}", self.entry.text())
    }
}

fn login_callback(gdata: Arc<GlobalData>, stack: Stack, login_box: LoginBox) {
    let Credentials {
        address,
        username,
        password,
    } = login_box.get_credentials();

    if address.is_empty() || username.is_empty() || password.is_empty() {
        return;
    }

    login_box.set_sensitive(false);

    let (sender, receiver) = async_channel::bounded::<Result<(), String>>(1);

    std::thread::spawn(move || {
        let password_hashed = get_password_hash(&password);

        let auth = Auth::new(address.clone(), username.clone(), password_hashed.clone());
        let result = auth.run().map_err(|e| e.to_string());

        match result {
            Ok(string) => match gdata.user_data.lock() {
                Ok(mut locked) => {
                    locked.address = address;
                    locked.user = username;
                    locked.password = password_hashed;
                    locked.token = string;
                    let _ = sender.send_blocking(Ok(()));
                }
                Err(err) => {
                    eprintln!("{}", err);
                    let _ = sender.send_blocking(Err(err.to_string()));
                }
            },
            Err(err) => {
                eprintln!("{}", err);
                let _ = sender.send_blocking(Err(err));
            }
        }
    });

    let stack = stack.clone();
    gtk4::glib::spawn_future_local(async move {
        if let Ok(result) = receiver.recv().await {
            match result {
                Ok(_) => {
                    stack.set_visible_child_name("main");
                    login_box.error.set_visible(false);
                }
                Err(err) => {
                    login_box.error.set_label(&err);
                    login_box.error.set_visible(true);
                }
            }
        }

        login_box.set_sensitive(true);
    });
}
