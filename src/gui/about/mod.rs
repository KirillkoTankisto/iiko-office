use gtk4::{AboutDialog, ApplicationWindow, License::Gpl20, gdk::Texture, glib::Bytes, prelude::*};

use crate::gui::{
    common::logo::LOGO,
    translation::{
        CurrentLanguage,
        Line::{COMMENT, SOURCE_CODE},
        translate,
    },
};

const PROGRAMNAME: &str = "iikoOffice";
const AUTHORS: &[&str] = &["Kirill Sergeev"];
const WEBSITE: &str = "https://github.com/KirillkoTankisto/iiko-office";

pub struct AboutPopup {
    dialog: AboutDialog,
}

impl AboutPopup {
    pub fn new(window: &ApplicationWindow, language: CurrentLanguage) -> Self {
        let logo_bytes = Bytes::from_static(LOGO);
        let logo = Texture::from_bytes(&logo_bytes).expect("invalid logo image");

        Self {
            dialog: AboutDialog::builder()
                .transient_for(window)
                .modal(true)
                .program_name(PROGRAMNAME)
                .comments(translate(language, COMMENT))
                .authors(AUTHORS)
                .license_type(Gpl20)
                .logo(&logo)
                .version(env!("CARGO_PKG_VERSION"))
                .website(WEBSITE)
                .website_label(translate(language, SOURCE_CODE))
                .build(),
        }
    }

    pub fn present(&self) {
        self.dialog.present();
    }
}
