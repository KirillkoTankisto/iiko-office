use gtk4::{gdk::Texture, glib::Bytes, prelude::*};

use crate::gui::{
    common::{
        logo::LOGO,
        modal::{Modal, closer, label, wrapped},
    },
    translation::{
        CurrentLanguage,
        Line::{ABOUT_COMMENT, ABOUT_SOURCE_CODE},
        translate,
    },
};

const PROGRAMNAME: &str = "iikoOffice";
const AUTHORS: &[&str] = &["Kirill Sergeev"];
const WEBSITE: &str = "https://github.com/KirillkoTankisto/iiko-office";

pub async fn show_about(window: &gtk4::ApplicationWindow, language: CurrentLanguage) {
    let modal = Modal::install(window);

    let logo = Texture::from_bytes(&Bytes::from_static(LOGO)).expect("invalid logo image");
    let image = gtk4::Image::from_paintable(Some(&logo));
    image.set_pixel_size(96);

    let name = label(PROGRAMNAME, &["title-1"]);
    let version = label(env!("CARGO_PKG_VERSION"), &["dim-label"]);
    let comment = wrapped(translate(language, ABOUT_COMMENT));
    let link = gtk4::LinkButton::with_label(WEBSITE, translate(language, ABOUT_SOURCE_CODE));
    let credits = label(&format!("{}\nGPL-2.0", AUTHORS.join(", ")), &["dim-label"]);
    credits.set_justify(gtk4::Justification::Center);
    let (close, closed) = closer(language);

    modal
        .show(
            24,
            &[
                image.upcast_ref(),
                name.upcast_ref(),
                version.upcast_ref(),
                comment.upcast_ref(),
                link.upcast_ref(),
                credits.upcast_ref(),
                close.upcast_ref(),
            ],
            closed,
        )
        .await;
}
