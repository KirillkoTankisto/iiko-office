use std::cell::Ref;

use gtk4::gdk::{ContentProvider, DragAction};
use gtk4::gio::ListStore;
use gtk4::glib;
use gtk4::glib::BoxedAnyObject;
use gtk4::{Align, DragSource, ScrolledWindow, SingleSelection, prelude::*};
use gtk4::{
    ColumnView, ColumnViewColumn, Label, ListItem, SignalListItemFactory, glib::object::Cast,
};

use std::marker::PhantomData;

#[derive(Clone, glib::Downgrade)]
pub struct AnyTable {
    column_view: ColumnView,
    store: ListStore,
    scrolled_window: ScrolledWindow,
}

impl AnyTable {
    pub fn new(expand: bool) -> Self {
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
            .hexpand(expand)
            .vexpand(true)
            .propagate_natural_width(!expand)
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

        let xalign: f32 = match align {
            Align::End => 1.0,
            Align::Center => 0.5,
            _ => 0.0,
        };

        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, item| {
            item.downcast_ref::<ListItem>().unwrap().set_child(Some(
                &Label::builder()
                    .halign(align)
                    .xalign(xalign)
                    .ellipsize(gtk4::pango::EllipsizeMode::End)
                    .max_width_chars(40)
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

    pub fn remove_columns(&self) {
        while let Some(column) = self.column_view.columns().item(0) {
            self.column_view
                .remove_column(column.downcast_ref::<ColumnViewColumn>().unwrap());
        }
    }

    pub fn connect<F>(&self, f: F)
    where
        F: Fn(&ColumnView, u32) + 'static,
    {
        self.column_view
            .connect_activate(move |column_view, row| f(column_view, row));
    }

    // sets dragging for the last added row
    pub fn set_row_drag<T, F>(&self, getter: F)
    where
        T: 'static,
        F: Fn(&T) -> String + 'static,
    {
        let columns = self.column_view.columns();
        let n = columns.n_items();
        if n == 0 {
            return;
        }
        let col = columns
            .item(n - 1)
            .unwrap()
            .downcast::<ColumnViewColumn>()
            .unwrap();
        let factory = col
            .factory()
            .unwrap()
            .downcast::<SignalListItemFactory>()
            .unwrap();

        let getter = std::rc::Rc::new(getter);

        factory.connect_setup(move |_, item| {
            let list_item = item.downcast_ref::<ListItem>().unwrap();
            let Some(child) = list_item.child() else {
                return;
            };

            let drag_source = DragSource::new();
            drag_source.set_actions(DragAction::COPY);

            let weak_item = list_item.downgrade();
            let getter = getter.clone();
            drag_source.connect_prepare(move |_, _, _| {
                let list_item = weak_item.upgrade()?;
                let obj = list_item.item()?.downcast::<BoxedAnyObject>().ok()?;
                let value: Ref<T> = obj.borrow();
                let payload = (*getter)(&value);
                Some(ContentProvider::for_value(&payload.to_value()))
            });

            child.add_controller(drag_source);
        });
    }
}

impl Default for AnyTable {
    fn default() -> Self {
        Self::new(true)
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
