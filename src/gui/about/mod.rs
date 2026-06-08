use std::sync::Arc;

use gtk4::{AboutDialog, ApplicationWindow, License::Gpl20, gdk::Texture, glib::Bytes, prelude::*};

use crate::gui::{
    GlobalData,
    translation::{
        Line::{COMMENT, SOURCE_CODE},
        translate,
    },
};

const PROGRAMNAME: &str = "iikoOffice";
const AUTHORS: &[&str] = &["Kirill Sergeev"];
const WEBSITE: &str = "https://github.com/KirillkoTankisto/iiko-office";

pub fn create_about(window: &ApplicationWindow, gdata: Arc<GlobalData>) {
    let logo_bytes = Bytes::from_static(include_bytes!("../../assets/logo.png"));
    let logo = Texture::from_bytes(&logo_bytes).expect("invalid logo image");

    let dialog = AboutDialog::builder()
        .transient_for(window)
        .modal(true)
        .program_name(PROGRAMNAME)
        .comments(translate(gdata.language.clone(), COMMENT))
        .authors(AUTHORS)
        .license_type(Gpl20)
        .logo(&logo)
        .version(env!("CARGO_PKG_VERSION"))
        .website(WEBSITE)
        .website_label(translate(gdata.language.clone(), SOURCE_CODE))
        .build();

    dialog.present();
}
