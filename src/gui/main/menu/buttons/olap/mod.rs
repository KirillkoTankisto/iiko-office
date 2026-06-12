use std::sync::Arc;

use gtk4::{Button, Notebook, prelude::ButtonExt};

use crate::gui::{GlobalData, main::menu::tabs::olap::create_olap};

pub fn create_olap_button(gdata: Arc<GlobalData>, view: &Notebook) -> Button
{
    let button = Button::with_label("OLAP Report");
    let view = view.clone();
    button.connect_clicked(move |_| {
        create_olap(gdata.clone(), &view);
    });
    button
}
