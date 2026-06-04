pub mod cashshifts;

use gtk4::Box;
use gtk4::Button;
use gtk4::Label;
use gtk4::Notebook;
use gtk4::prelude::*;

pub fn add_tab<'a, S: Into<&'a str>>(view: &Notebook, tab: &Box, label: S) -> Box {
    let tab_box = Box::new(gtk4::Orientation::Horizontal, 16);

    let label = Label::new(Some(label.into()));
    let close_btn = Button::builder()
        .icon_name("window-close-symbolic")
        .halign(gtk4::Align::End)
        .build();

    tab_box.append(&label);
    tab_box.append(&close_btn);

    let view = view.clone();
    let tab = tab.clone();
    close_btn.connect_clicked(move |_| {
        if let Some(page_number) = view.page_num(&tab) {
            view.remove_page(Some(page_number));
        }
    });

    tab_box
}
