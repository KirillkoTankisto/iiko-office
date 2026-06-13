use std::cell::Ref;

use gtk4::glib::BoxedAnyObject;
use gtk4::{Align, prelude::*};
use gtk4::{
    ColumnView, ColumnViewColumn, Label, ListItem, SignalListItemFactory, glib::object::Cast,
};

pub fn add_col<T, F>(cv: &ColumnView, title: &str, align: Align, expand: bool, getter: F)
where
    T: 'static,
    F: Fn(&T) -> String + 'static,
{
    let factory = SignalListItemFactory::new();
    factory.connect_setup(move |_, item| {
        item.downcast_ref::<ListItem>().unwrap().set_child(Some(
            &Label::builder()
                .halign(align)
                .margin_start(4)
                .margin_end(4)
                .margin_top(4)
                .margin_bottom(4)
                .build(),
        ));
    });
    factory.connect_bind(move |_, item| {
        let item = item.downcast_ref::<ListItem>().unwrap();
        let label = item.child().unwrap().downcast::<Label>().unwrap();
        let obj = item.item().unwrap().downcast::<BoxedAnyObject>().unwrap();
        let value: Ref<T> = obj.borrow();
        label.set_label(&getter(&value));
    });
    let col = ColumnViewColumn::new(Some(title), Some(factory));
    col.set_resizable(true);
    col.set_expand(expand);

    cv.set_halign(Align::Fill);
    cv.append_column(&col);
}
