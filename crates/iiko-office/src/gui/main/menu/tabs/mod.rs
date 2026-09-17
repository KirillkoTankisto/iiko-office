use std::sync::Arc;

use crate::gui::{GlobalData, main::menu::view::MainView};

pub mod cashshifts;
pub mod cashshifts_payments;
pub mod employees;
pub mod olap_reports;

pub trait AnyTab {
    fn title(&self, gdata: &GlobalData) -> &str;

    fn build(&self, gdata: Arc<GlobalData>, view: &MainView) -> gtk4::Widget;
}
