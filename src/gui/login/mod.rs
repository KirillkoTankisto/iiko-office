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

use crate::api::auth::*;
use crate::gui::GlobalData;
use crate::gui::translation::Line::ADDRESS;
use crate::gui::translation::Line::LOGIN;
use crate::gui::translation::Line::PASSWORD;
use crate::gui::translation::Line::USERNAME;
use crate::gui::translation::translate;

pub fn create_login(gdata: Arc<GlobalData>, stack: Stack) -> Box {
    let login_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_start(64)
        .margin_end(64)
        .margin_bottom(16)
        .margin_top(16)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    login_box.append(&Label::new(Some(translate(
        gdata.language.clone(),
        ADDRESS,
    ))));
    let address_box = AddressBox::new();
    login_box.append(&address_box.root);

    login_box.append(&Label::new(Some(translate(
        gdata.language.clone(),
        USERNAME,
    ))));
    let username_entry = Entry::builder().width_chars(32).build();
    login_box.append(&username_entry);

    login_box.append(&Label::new(Some(translate(
        gdata.language.clone(),
        PASSWORD,
    ))));
    let password_entry = PasswordEntry::builder().width_chars(32).build();
    login_box.append(&password_entry);

    let button = Button::builder()
        .label(translate(gdata.language.clone(), LOGIN))
        .margin_top(24)
        .width_request(360)
        .halign(Align::Fill)
        .build();

    button.connect_clicked(move |button| {
        login_callback(
            gdata.clone(),
            &stack,
            button,
            &address_box,
            &username_entry,
            &password_entry,
        );
    });
    login_box.append(&button);

    login_box
}

struct AddressBox {
    root: Box,
    dropdown: DropDown,
    entry: Entry,
}

impl AddressBox {
    fn new() -> Self {
        let address_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        let dropdown = DropDown::from_strings(&["https://", "http://"]);
        dropdown.set_size_request(90, -1);
        let entry = Entry::builder().width_chars(32).build();

        address_box.append(&dropdown);
        address_box.append(&entry);

        Self {
            root: address_box,
            dropdown,
            entry,
        }
    }

    fn get_url(&self) -> String {
        let scheme = self
            .dropdown
            .selected_item()
            .and_downcast::<gtk4::StringObject>()
            .map(|s| s.string().to_string())
            .unwrap_or("https".into());
        let address = self.entry.text().to_string();
        let mut string = String::with_capacity(scheme.len() + address.len() + 1);

        string.push_str(&scheme);
        string.push_str(&address);

        string
    }
}

fn login_callback(
    gdata: Arc<GlobalData>,
    stack: &Stack,
    button: &Button,
    url: &AddressBox,
    username: &Entry,
    password: &PasswordEntry,
) {
    let url_text = url.get_url();
    let username_text = username.text().to_string();
    let password_text = password.text().to_string();
    if url_text.is_empty() || username_text.is_empty() || password_text.is_empty() {
        return;
    }

    button.set_sensitive(false);

    let (sender, receiver) = async_channel::bounded::<bool>(1);

    std::thread::spawn(move || {
        let password_hashed = get_password_hash(&password_text);

        let auth = Auth {
            address: url_text.clone(),
            user: username_text.clone(),
            pass: password_hashed.clone(),
        };
        let result = auth.run();
        if let Ok(token) = result
            && let Ok(mut locked) = gdata.user_data.lock()
        {
            locked.address = url_text;
            locked.user = username_text;
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
    let button = button.clone();
    gtk4::glib::spawn_future_local(async move {
        if let Ok(result) = receiver.recv().await
            && result
        {
            stack.set_visible_child_name("main");
        }

        button.set_sensitive(true);
    });
}
