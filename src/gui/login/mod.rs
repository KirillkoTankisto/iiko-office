use std::sync::Arc;

use gtk4::Button;
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
    let address_entry = Entry::builder().width_chars(32).build();
    login_box.append(&address_entry);

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
        .halign(Align::Center)
        .build();

    button.connect_clicked(move |button| {
        login_callback(
            gdata.clone(),
            &stack,
            button,
            &address_entry,
            &username_entry,
            &password_entry,
        );
    });
    login_box.append(&button);

    login_box
}

fn login_callback(
    gdata: Arc<GlobalData>,
    stack: &Stack,
    button: &Button,
    address: &Entry,
    username: &Entry,
    password: &PasswordEntry,
) {
    let address_text = address.text().to_string();
    let username_text = username.text().to_string();
    let password_text = password.text().to_string();
    if address_text.is_empty() || username_text.is_empty() || password_text.is_empty() {
        return;
    }

    button.set_sensitive(false);

    let (sender, receiver) = async_channel::bounded::<bool>(1);

    std::thread::spawn(move || {
        let password_hashed = get_password_hash(&password_text);

        let auth = Auth {
            address: address_text.clone(),
            user: username_text.clone(),
            pass: password_hashed.clone(),
        };
        let result = auth.run();
        if let Ok(token) = result
            && let Ok(mut locked) = gdata.user_data.lock()
        {
            locked.address = address_text;
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
