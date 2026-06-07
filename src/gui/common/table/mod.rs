use std::cell::Ref;

use gtk4::glib::BoxedAnyObject;
use gtk4::prelude::*;
use gtk4::{
    ColumnView, ColumnViewColumn, Label, ListItem, SignalListItemFactory, glib::object::Cast,
};

pub fn add_col<T, F>(cv: &ColumnView, title: &str, getter: F)
where
    T: 'static,
    F: Fn(&T) -> String + 'static,
{
    let factory = SignalListItemFactory::new();
    factory.connect_setup(|_, item| {
        item.downcast_ref::<ListItem>()
            .unwrap()
            .set_child(Some(&Label::new(None)));
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
    cv.append_column(&col);
}
