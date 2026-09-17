use gtk4::{Widget, glib, prelude::*};

#[derive(glib::Downgrade)]
pub struct AnyBox {
    root: gtk4::Box,
}

const SPACING: i32 = 8;

impl AnyBox {
    pub fn vertical() -> Self {
        Self::with_orientation(gtk4::Orientation::Vertical)
    }

    pub fn horizontal() -> Self {
        Self::with_orientation(gtk4::Orientation::Horizontal)
    }

    fn with_orientation(orientation: gtk4::Orientation) -> Self {
        Self {
            root: gtk4::Box::new(orientation, SPACING),
        }
    }

    pub fn add(&self, widget: &impl IsA<Widget>) {
        self.root().append(widget);
    }

    pub fn add_widgets<const N: usize>(self, widgets: [&Widget; N]) -> Self {
        let root = self.root();
        for widget in widgets {
            root.append(widget);
        }
        self
    }

    pub fn add_widgets_vec(self, widgets: impl IntoIterator<Item = Widget>) -> Self {
        let root = self.root();
        for widget in widgets {
            root.append(&widget);
        }
        self
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.root
    }

    pub fn consume(self) -> gtk4::Box {
        self.root
    }

    pub fn margin(self, margin: i32) -> Self {
        let root = self.root();
        root.set_margin_start(margin);
        root.set_margin_end(margin);
        root.set_margin_top(margin);
        root.set_margin_bottom(margin);

        self
    }

    pub fn align(self, align: gtk4::Align) -> Self {
        let root = self.root();
        root.set_halign(align);
        root.set_valign(align);

        self
    }

    pub fn width_request(self, value: i32) -> Self {
        let root = self.root();
        root.set_width_request(value);

        self
    }

    pub fn height_request(self, value: i32) -> Self {
        let root = self.root();
        root.set_height_request(value);

        self
    }

    pub fn set_visible(&self, value: bool) {
        self.root().set_visible(value);
    }
}

impl Default for AnyBox {
    fn default() -> Self {
        Self::horizontal()
    }
}
