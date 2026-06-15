use std::cell::Ref;

use gtk4::gio::ListStore;
use gtk4::glib::BoxedAnyObject;
use gtk4::{Align, ScrolledWindow, SingleSelection, prelude::*};
use gtk4::{
    ColumnView, ColumnViewColumn, Label, ListItem, SignalListItemFactory, glib::object::Cast,
};

use std::marker::PhantomData;

#[derive(Clone)]
pub struct AnyTable {
    column_view: ColumnView,
    store: ListStore,
    scrolled_window: ScrolledWindow,
}

impl AnyTable {
    pub fn new() -> Self {
        let store = ListStore::new::<BoxedAnyObject>();
        let selection = SingleSelection::new(Some(store.clone()));
        let column_view = ColumnView::builder()
            .model(&selection)
            .hexpand(true)
            .halign(Align::Fill)
            .show_column_separators(true)
            .show_row_separators(true)
            .build();

        let scrolled_window = ScrolledWindow::builder()
            .child(&column_view)
            .halign(Align::Fill)
            .valign(Align::Fill)
            .hexpand(true)
            .vexpand(true)
            .build();
        Self {
            column_view,
            store,
            scrolled_window,
        }
    }

    pub fn add_column<T, F>(&self, column: AnyTableColumn<'_, T, F>)
    where
        T: 'static,
        F: Fn(&T) -> String + 'static,
    {
        let AnyTableColumn {
            title,
            align,
            expand,
            getter,
            ..
        } = column;

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
            label.set_label(&getter(&value)); // closure call, not a method
        });

        let col = ColumnViewColumn::new(Some(title), Some(factory));
        col.set_resizable(true);
        col.set_expand(expand);
        self.column_view.append_column(&col);
    }

    pub fn present(&self) -> &ScrolledWindow {
        &self.scrolled_window
    }

    pub fn add_object(&self, object: &BoxedAnyObject) {
        self.store.append(object);
    }

    pub fn clear_table(&self) {
        self.store.remove_all();
    }

    pub fn connect<F>(&self, f: F)
    where
        F: Fn(&ColumnView, u32) + 'static,
    {
        self.column_view
            .connect_activate(move |column_view, row| f(column_view, row));
    }
}

impl Default for AnyTable {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AnyTableColumn<'a, T, F> {
    title: &'a str,
    align: Align,
    expand: bool,
    getter: F,
    _marker: PhantomData<fn(&T)>,
}

impl<'a, T, F> AnyTableColumn<'a, T, F>
where
    F: Fn(&T) -> String,
{
    pub fn new(title: &'a str, align: Align, expand: bool, getter: F) -> Self {
        Self {
            title,
            align,
            expand,
            getter,
            _marker: PhantomData,
        }
    }
}
