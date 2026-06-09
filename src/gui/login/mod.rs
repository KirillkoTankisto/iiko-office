use std::sync::Arc;

use gtk4::Button;
use gtk4::DropDown;
use gtk4::prelude::*;

use gtk4::Align;
use gtk4::Box;
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
    pub root: Box,
    address: AddressBox,
    username: Entry,
    password: PasswordEntry,
    button: Button,
}

impl LoginBox {
    pub fn new(gdata: Arc<GlobalData>, stack: Stack) -> Self {
        let root = Box::builder()
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

        let address_box = AddressBox::new();
        let username_entry = Entry::builder().width_chars(32).build();
        let password_entry = PasswordEntry::builder().width_chars(32).build();

        root.append(&label(ADDRESS));
        root.append(&address_box.root);
        root.append(&label(USERNAME));
        root.append(&username_entry);
        root.append(&label(PASSWORD));
        root.append(&password_entry);

        let button = Button::builder()
            .label(translate(gdata.language(), LOGIN))
            .margin_top(24)
            .width_request(360)
            .halign(Align::Fill)
            .build();

        root.append(&button);

        let login_box = Self {
            root,
            address: address_box,
            username: username_entry,
            password: password_entry,
            button,
        };

        login_box.button.connect_clicked(glib::clone!(
            #[strong]
            gdata,
            #[weak]
            stack,
            #[weak(rename_to = login_box)]
            login_box,
            move |_| {
                login_callback(gdata.clone(), stack, login_box);
            }
        ));

        login_box
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
    root: Box,
    dropdown: DropDown,
    entry: Entry,
}

impl AddressBox {
    fn new() -> Self {
        let root = Box::builder()
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
    let creds = login_box.get_credentials();
    if creds.address.is_empty() || creds.username.is_empty() || creds.password.is_empty() {
        return;
    }

    login_box.set_sensitive(false);

    let (sender, receiver) = async_channel::bounded::<bool>(1);

    std::thread::spawn(move || {
        let Credentials {
            address,
            username,
            password,
        } = creds;

        let password_hashed = get_password_hash(&password);

        let auth = Auth {
            address: address.clone(),
            user: username.clone(),
            pass: password_hashed.clone(),
        };
        let result = auth.run();
        if let Ok(token) = result
            && let Ok(mut locked) = gdata.user_data.lock()
        {
            locked.address = address;
            locked.user = username;
            locked.password = password_hashed;
            locked.token = token;

            println!("Login success!");
            let _ = sender.send_blocking(true);
        } else {
            println!("Login failure!");
            let _ = sender.send_blocking(false);
        }
    });

    let stack = stack.clone();
    gtk4::glib::spawn_future_local(async move {
        if let Ok(result) = receiver.recv().await
            && result
        {
            stack.set_visible_child_name("main");
        }

        login_box.set_sensitive(true);
    });
}
