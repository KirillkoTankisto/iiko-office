use gtk4::{ApplicationWindow, prelude::*};

use crate::gui::translation::{CurrentLanguage, Line::CLOSE, translate};

pub struct Modal {
    overlay: gtk4::Overlay,
    content: Option<gtk4::Widget>,
}

impl Modal {
    pub fn install(window: &ApplicationWindow) -> Self {
        let overlay = gtk4::Overlay::new();
        let content = window.child();
        window.set_child(Some(&overlay));
        overlay.set_child(content.as_ref());
        Self { overlay, content }
    }

    pub async fn show(
        &self,
        margin: i32,
        widgets: &[&gtk4::Widget],
        closed: async_channel::Receiver<()>,
    ) {
        let inner = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        inner.set_margin_top(margin);
        inner.set_margin_bottom(margin);
        inner.set_margin_start(margin);
        inner.set_margin_end(margin);
        for widget in widgets {
            inner.append(*widget);
        }

        let card = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        card.add_css_class("background");
        card.add_css_class("frame");
        card.set_halign(gtk4::Align::Center);
        card.set_valign(gtk4::Align::Center);
        card.append(&inner);

        self.set_content_sensitive(false);
        self.overlay.add_overlay(&card);
        if let Some(last) = widgets.last() {
            last.grab_focus();
        }

        let _ = closed.recv().await;

        self.overlay.remove_overlay(&card);
        self.set_content_sensitive(true);
    }

    fn set_content_sensitive(&self, state: bool) {
        if let Some(content) = &self.content {
            content.set_sensitive(state);
        }
    }
}

pub fn closer(language: CurrentLanguage) -> (gtk4::Button, async_channel::Receiver<()>) {
    let (tx, rx) = async_channel::bounded(1);
    let button = gtk4::Button::with_label(translate(language, CLOSE));
    button.connect_clicked(move |_| {
        let _ = tx.try_send(());
    });
    (button, rx)
}

pub fn label(text: &str, classes: &[&str]) -> gtk4::Label {
    let label = gtk4::Label::new(Some(text));
    for class in classes {
        label.add_css_class(class);
    }
    label
}

pub fn wrapped(text: &str) -> gtk4::Label {
    let label = gtk4::Label::new(Some(text));
    label.set_wrap(true);
    label.set_max_width_chars(40);
    label
}
