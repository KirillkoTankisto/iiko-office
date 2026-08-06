use gtk4::{gdk::Texture, glib::Bytes};

const LOGO: &[u8] = include_bytes!("../../../assets/logo.png");

/// The application logo, rendered at `size` pixels.
pub fn logo_image(size: i32) -> gtk4::Image {
    let texture = Texture::from_bytes(&Bytes::from_static(LOGO)).expect("invalid logo image");
    let image = gtk4::Image::from_paintable(Some(&texture));
    image.set_pixel_size(size);

    image
}
