use std::sync::Arc;

use gtk4::{Notebook, Orientation::Vertical};

use crate::gui::{GlobalData, main::menu::tabs::add_tab};

pub fn create_olap(gdata: Arc<GlobalData>, view: &Notebook)
{
    let olap_box = gtk4::Box::new(Vertical, 8);

    let olap_tab = add_tab(view, &olap_box, "OLAP Report");

    view.append_page(&olap_box, Some(&olap_tab));
}
