use gtk4::Notebook;

pub fn create_view() -> Notebook {
    Notebook::builder()
        .scrollable(true)
        .hexpand(true)
        .vexpand(true)
        .margin_end(16)
        .margin_bottom(16)
        .build()
}
