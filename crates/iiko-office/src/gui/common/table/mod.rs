use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use gtk4::gdk::{ContentProvider, DragAction};
use gtk4::gio::ListStore;
use gtk4::glib::BoxedAnyObject;
use gtk4::{
    Align, DragSource, FilterChange, FilterListModel, ScrolledWindow, SearchEntry, SingleSelection,
    prelude::*,
};
use gtk4::{
    ColumnView, ColumnViewColumn, Label, ListItem, SignalListItemFactory, glib::object::Cast,
};
use gtk4::{CustomFilter, glib};
use iiko_api::olap::OlapTable;

use std::marker::PhantomData;

use crate::gui::translation::CurrentLanguage;

type SearchGetter = Box<dyn Fn(&BoxedAnyObject) -> String>;

#[derive(Clone, glib::Downgrade)]
pub struct AnyTable {
    column_view: ColumnView,
    store: ListStore,
    scrolled_window: ScrolledWindow,
    filter: CustomFilter,
    query: Rc<RefCell<String>>,
    search_getters: Rc<RefCell<Vec<SearchGetter>>>,
}

const MIN_WIDTH_CHARS: i32 = 12;
const MAX_WIDTH_CHARS: i32 = 40;

#[derive(Clone, Copy)]
pub enum OlapLayout {
    /// Flat listing: every column is a field id; alignment inferred from data.
    Flat,
    /// Pivot: the first `key_count` columns are field ids, the rest are
    /// category values holding aggregates.
    Pivot { key_count: usize },
}

impl AnyTable {
    pub fn new(expand: bool) -> Self {
        let store = ListStore::new::<BoxedAnyObject>();
        let query = Rc::new(RefCell::new(String::new()));
        let search_getters: Rc<RefCell<Vec<SearchGetter>>> = Rc::new(RefCell::new(Vec::new()));

        let filter = CustomFilter::new({
            let query = query.clone();
            let search_getters = search_getters.clone();
            move |obj| {
                let query = query.borrow();
                if query.is_empty() {
                    return true;
                }
                let getters = search_getters.borrow();
                if getters.is_empty() {
                    return true;
                }
                let Some(obj) = obj.downcast_ref::<BoxedAnyObject>() else {
                    return true;
                };
                let needle = query.to_lowercase();
                getters
                    .iter()
                    .any(|getter| getter(obj).to_lowercase().contains(&needle))
            }
        });

        let filter_model = FilterListModel::new(Some(store.clone()), Some(filter.clone()));

        let selection = SingleSelection::new(Some(filter_model));
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
            filter,
            query,
            search_getters,
        }
    }

    pub fn set_olap_table(
        &self,
        olap_table: OlapTable,
        // Field name resolution
        id_to_name: &HashMap<String, String>,
        layout: OlapLayout,
    ) {
        self.clear_table();
        self.remove_columns();

        let OlapTable { columns, rows } = olap_table;

        let key_count = match layout {
            OlapLayout::Flat => columns.len(),
            OlapLayout::Pivot { key_count } => key_count,
        };

        for (index, column) in columns.iter().enumerate() {
            let title = if index < key_count {
                id_to_name
                    .get(column)
                    .cloned()
                    .unwrap_or_else(|| column.clone())
            } else {
                column.clone()
            };

            let align = match layout {
                OlapLayout::Pivot { .. } => {
                    if index < key_count {
                        Align::Start
                    } else {
                        Align::End
                    }
                }
                OlapLayout::Flat if column_is_numeric(&rows, index) => Align::End,
                OlapLayout::Flat => Align::Start,
            };

            self.add_column(AnyTableColumn::new(
                &title,
                align,
                false,
                true,
                move |row: &Vec<String>| row.get(index).cloned().unwrap_or_default(),
            ));
        }

        for row in rows {
            self.add_object(&BoxedAnyObject::new(row));
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
            searchable,
            getter,
            ..
        } = column;

        let xalign: f32 = match align {
            Align::End => 1.0,
            Align::Center => 0.5,
            _ => 0.0,
        };

        let getter = Rc::new(getter);

        if searchable {
            let getter = getter.clone();
            self.search_getters.borrow_mut().push(Box::new(move |obj| {
                let value: Ref<T> = obj.borrow();
                getter(&value)
            }));
        }

        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, item| {
            let attrs = gtk4::pango::AttrList::new();
            attrs.insert(gtk4::pango::AttrFloat::new_scale(0.8333));
            item.downcast_ref::<ListItem>().unwrap().set_child(Some(
                &Label::builder()
                    .halign(align)
                    .xalign(xalign)
                    .ellipsize(gtk4::pango::EllipsizeMode::End)
                    .width_chars(MIN_WIDTH_CHARS)
                    .max_width_chars(MAX_WIDTH_CHARS)
                    .attributes(&attrs)
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
        self.search_getters.borrow_mut().clear();
    }

    pub fn connect<F>(&self, f: F)
    where
        F: Fn(&ColumnView, u32) + 'static,
    {
        self.column_view
            .connect_activate(move |column_view, row| f(column_view, row));
    }

    // sets dragging for the last added column
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

        let getter = Rc::new(getter);

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

    pub fn get_items<T: Clone + 'static>(&self) -> Vec<T> {
        self.store
            .iter::<BoxedAnyObject>()
            .filter_map(Result::ok)
            .map(|obj| obj.borrow::<T>().clone())
            .collect()
    }

    pub fn search_entry(&self) -> SearchEntry {
        let entry = SearchEntry::new();
        entry.connect_search_changed(glib::clone!(
            #[weak(rename_to = table)]
            self,
            move |entry| {
                table.set_search_query(&entry.text());
            }
        ));
        entry
    }

    pub fn set_search_query(&self, text: &str) {
        let old = self.query.replace(text.to_string());

        let new_lowercase = text.to_lowercase();
        let old_lowercase = old.to_lowercase();
        if new_lowercase == old_lowercase {
            return;
        }

        let change = if new_lowercase.contains(&old_lowercase) {
            FilterChange::MoreStrict
        } else if old_lowercase.contains(&new_lowercase) {
            FilterChange::LessStrict
        } else {
            FilterChange::Different
        };

        self.filter.changed(change);
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
    searchable: bool,
    getter: F,
    _marker: PhantomData<fn(&T)>,
}

impl<'a, T, F> AnyTableColumn<'a, T, F>
where
    F: Fn(&T) -> String,
{
    pub fn new(title: &'a str, align: Align, expand: bool, searchable: bool, getter: F) -> Self {
        Self {
            title,
            align,
            expand,
            searchable,
            getter,
            _marker: PhantomData,
        }
    }
}

pub trait AsTable {
    fn as_table(language: CurrentLanguage) -> AnyTable;
}

fn column_is_numeric(rows: &[Vec<String>], index: usize) -> bool {
    rows.iter()
        .filter_map(|row| row.get(index))
        .find(|cell| !cell.is_empty())
        .is_some_and(|cell| cell.parse::<f64>().is_ok())
}
