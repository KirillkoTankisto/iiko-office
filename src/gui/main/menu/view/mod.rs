use gtk4::Button;
use gtk4::Label;
use gtk4::Notebook;
use gtk4::Widget;
use gtk4::glib;

use gtk4::prelude::*;

#[derive(Clone)]
pub struct MainView {
    root: Notebook,
}

impl MainView {
    pub fn new() -> Self {
        Self {
            root: Notebook::builder()
                .scrollable(true)
                .hexpand(true)
                .vexpand(true)
                .margin_end(16)
                .margin_bottom(16)
                .build(),
        }
    }

    pub fn present(&self) -> &Notebook {
        &self.root
    }

    pub fn add_tab(&self, tab: &Widget, label: &str) {
        let tab_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);

        let label = Label::new(Some(label));
        let close_btn = Button::builder()
            .icon_name("window-close-symbolic")
            .has_frame(false)
            .halign(gtk4::Align::End)
            .build();

        tab_box.append(&label);
        tab_box.append(&close_btn);

        let notebook = self.present().clone();
        close_btn.connect_clicked(glib::clone!(
            #[weak]
            tab,
            move |_| {
                if let Some(page_number) = notebook.page_num(&tab) {
                    notebook.remove_page(Some(page_number));
                }
            }
        ));

        self.root.append_page(tab, Some(&tab_box));
    }
}
