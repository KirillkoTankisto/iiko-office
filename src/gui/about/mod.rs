use gtk4::{AboutDialog, ApplicationWindow, License::Gpl20, gdk::Texture, glib::Bytes, prelude::*};

const PROGRAMNAME: &str = "iikoOffice";
const AUTHORS: &[&str] = &["Kirill Sergeev"];
const WEBSITE: &str = "https://github.com/KirillkoTankisto/iiko-office";

pub fn create_about(window: &ApplicationWindow) {
    let logo_bytes = Bytes::from_static(include_bytes!("../../assets/logo.png"));
    let logo = Texture::from_bytes(&logo_bytes).expect("invalid logo image");

    let dialog = AboutDialog::builder()
        .transient_for(window)
        .modal(true)
        .program_name(PROGRAMNAME)
        .comments("iikoOffice Open-Source alternative for Linux and macOS")
        .authors(AUTHORS)
        .license_type(Gpl20)
        .logo(&logo)
        .version(env!("CARGO_PKG_VERSION"))
        .website(WEBSITE)
        .website_label("Source code")
        .build();

    dialog.present();
}
