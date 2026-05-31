use gtk4::{Box, Button, Label, Notebook, prelude::*};

use crate::gui::common::datepicker::DatePicker;

pub fn create_cashshifts_button(view: &Notebook) -> Button {
    let button = Button::with_label("Cash Shifts");

    let view = view.clone();
    button.connect_clicked(move |_| {
        create_cashshifts(&view);
    });

    return button;
}

fn create_cashshifts(view: &Notebook) {
    let cashshifts_box = Box::builder()
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let grid = gtk4::Grid::builder()
        .column_spacing(8)
        .row_spacing(8)
        .build();

    let date_from = DatePicker::new("Date From");
    let date_to = DatePicker::new("Date To");

    date_from.attach_to(&grid, 0);
    date_to.attach_to(&grid, 1);

    cashshifts_box.append(&grid);

    view.append_page(&cashshifts_box, Some(&Label::new("Cash Shifts".into())));

    return;
}
