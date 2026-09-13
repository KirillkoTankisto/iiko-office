use gtk4::prelude::*;
use gtk4::{Frame, Widget};

use crate::gui::translation::{CurrentLanguage, Line, translate};

pub fn frame(lang: CurrentLanguage, line: Line, widget: &impl IsA<Widget>) -> Frame {
    let frame = Frame::builder().label(translate(lang, line)).build();
    frame.set_child(Some(widget));
    frame
}
