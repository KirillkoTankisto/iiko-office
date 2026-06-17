pub const LOGO: &[u8] = include_bytes!("../../../assets/logo.png");

pub fn get_logo_image() -> gtk4::Image {
    let logo = gtk4::gdk::Texture::from_bytes(&gtk4::glib::Bytes::from_static(LOGO)).unwrap();
    let image = gtk4::Image::from_paintable(Some(&logo));
    image.set_pixel_size(128);

    image
}
