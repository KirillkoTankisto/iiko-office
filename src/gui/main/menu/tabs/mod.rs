use std::sync::Arc;

use crate::gui::{GlobalData, main::menu::view::MainView};

pub mod cashshifts;
pub mod cashshifts_payments;

pub trait AnyTab {
    fn title(&self, gdata: &GlobalData) -> &str;

    fn build(&self, gdata: Arc<GlobalData>, view: &MainView) -> gtk4::Widget;
}

pub fn open_tab(tab: &dyn AnyTab, gdata: Arc<GlobalData>, view: &MainView) {
    let title = tab.title(&gdata);
    let content = tab.build(gdata, view);
    view.add_tab(&content, title);
}
