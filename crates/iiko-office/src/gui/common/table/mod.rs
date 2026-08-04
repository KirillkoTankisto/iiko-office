use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use gtk4::gdk::{ContentProvider, DragAction};
use gtk4::gio::ListStore;
use gtk4::glib::{self, BoxedAnyObject, object::Cast};
use gtk4::pango::{AttrFloat, AttrInt, AttrList, EllipsizeMode, Weight};
use gtk4::{
    Align, ColumnView, ColumnViewColumn, CustomFilter, DragSource, FilterChange, FilterListModel,
    Label, ListItem, ScrolledWindow, SearchEntry, SignalListItemFactory, SingleSelection,
    prelude::*,
};
use iiko_api::olap::{OlapRowKind, OlapTable};

use crate::gui::translation::CurrentLanguage;

type SearchGetter = Box<dyn Fn(&BoxedAnyObject) -> String>;

const MIN_WIDTH_CHARS: i32 = 12;
const MAX_WIDTH_CHARS: i32 = 40;
const INDENT_PX: i32 = 12;

#[derive(Clone, glib::Downgrade)]
pub struct AnyTable {
    column_view: ColumnView,
    store: ListStore,
    scrolled_window: ScrolledWindow,
    filter: CustomFilter,
    query: Rc<RefCell<String>>,
    search_getters: Rc<RefCell<Vec<SearchGetter>>>,
}

#[derive(Clone, Copy)]
pub enum OlapLayout {
    Pivot { key_count: usize },
    Grouped { key_count: usize },
}

impl OlapLayout {
    fn key_count(self, column_count: usize) -> usize {
        match self {
            OlapLayout::Pivot { key_count } | OlapLayout::Grouped { key_count } => {
                key_count.min(column_count)
            }
        }
    }
}

/// One row of a rendered OLAP table.
pub struct OlapRow {
    /// What is printed to the cell
    pub cells: Vec<String>,
    /// Actual name of a cell which
    /// is used when searching for
    /// same entry names
    pub full: Vec<String>,
    pub kind: OlapRowKind,
}

#[derive(Clone, Copy, Default)]
pub struct CellStyle {
    pub bold: bool,
    /// Indent in pixels
    pub indent: i32,
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
                let needle = query.borrow();
                let getters = search_getters.borrow();
                let Some(obj) = obj.downcast_ref::<BoxedAnyObject>() else {
                    return true;
                };
                needle.is_empty()
                    || getters.is_empty()
                    || getters
                        .iter()
                        .any(|getter| getter(obj).to_lowercase().contains(&*needle))
            }
        });

        let column_view = ColumnView::builder()
            .model(&SingleSelection::new(Some(FilterListModel::new(
                Some(store.clone()),
                Some(filter.clone()),
            ))))
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

        let OlapTable {
            columns,
            rows,
            row_kinds,
            ..
        } = olap_table;

        let key_count = layout.key_count(columns.len());
        let is_pivot = matches!(layout, OlapLayout::Pivot { .. });
        let resolver = TitleResolver::new(id_to_name);

        for (index, column) in columns.iter().enumerate() {
            let is_key = index < key_count;

            let title = if is_key || !is_pivot {
                resolver.title(column)
            } else {
                column.clone()
            };

            let align = if !is_key && (is_pivot || column_is_numeric(&rows, &row_kinds, index)) {
                Align::End
            } else {
                Align::Start
            };

            let query = self.query.clone();

            self.add_column(
                AnyTableColumn::new(&title, align, move |row: &OlapRow| {
                    let source = if is_key && !query.borrow().is_empty() {
                        &row.full
                    } else {
                        &row.cells
                    };
                    source.get(index).cloned().unwrap_or_default()
                })
                .style(move |row: &OlapRow| CellStyle {
                    bold: !matches!(row.kind, OlapRowKind::Data),
                    indent: match row.kind {
                        OlapRowKind::Subtotal { level } if is_key => level as i32,
                        _ => 0,
                    },
                })
                .searchable(),
            );
        }

        let mut carry = vec![String::new(); columns.len()];

        for (cells, kind) in rows.into_iter().zip(row_kinds) {
            let mut full = cells.clone();
            if matches!(kind, OlapRowKind::Data) {
                for (slot, value) in carry.iter_mut().zip(full.iter_mut()).take(key_count) {
                    if value.is_empty() {
                        value.clone_from(slot);
                    } else {
                        slot.clone_from(value);
                    }
                }
            }
            self.add_object(&BoxedAnyObject::new(OlapRow { cells, full, kind }));
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
            style,
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
            item.downcast_ref::<ListItem>().unwrap().set_child(Some(
                &Label::builder()
                    .halign(align)
                    .xalign(xalign)
                    .ellipsize(EllipsizeMode::End)
                    .width_chars(MIN_WIDTH_CHARS)
                    .max_width_chars(MAX_WIDTH_CHARS)
                    .build(),
            ));
        });
        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            let label = item.child().unwrap().downcast::<Label>().unwrap();
            let obj = item.item().unwrap().downcast::<BoxedAnyObject>().unwrap();
            let value: Ref<T> = obj.borrow();
            label.set_label(&getter(&value)); // closure call, not a method

            // Widgets are recycled, so every attribute is set on every bind —
            // otherwise a subtotal's bold leaks onto whatever row reuses it.
            let style = style.as_ref().map(|f| f(&value)).unwrap_or_default();
            let attrs = AttrList::new();
            attrs.insert(AttrFloat::new_scale(0.8333));
            if style.bold {
                attrs.insert(AttrInt::new_weight(Weight::Bold));
            }
            label.set_attributes(Some(&attrs));
            label.set_margin_start(style.indent * INDENT_PX);
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
        self.column_view.connect_activate(f);
    }

    // sets dragging for the last added column
    pub fn set_row_drag<T, F>(&self, getter: F)
    where
        T: 'static,
        F: Fn(&T) -> String + 'static,
    {
        let columns = self.column_view.columns();
        let Some(col) = columns.n_items().checked_sub(1).map(|last| {
            columns
                .item(last)
                .unwrap()
                .downcast::<ColumnViewColumn>()
                .unwrap()
        }) else {
            return;
        };
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
                let payload = getter(&value);
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
        let new = text.to_lowercase();
        let old = self.query.replace(new.clone());

        if new == old {
            return;
        }

        self.filter.changed(if new.contains(&old) {
            FilterChange::MoreStrict
        } else if old.contains(&new) {
            FilterChange::LessStrict
        } else {
            FilterChange::Different
        });
    }
}

type Styler<T> = dyn Fn(&T) -> CellStyle + 'static;

pub struct AnyTableColumn<'a, T: 'static, F> {
    title: &'a str,
    align: Align,
    expand: bool,
    searchable: bool,
    getter: F,
    style: Option<Box<Styler<T>>>,
}

impl<'a, T, F> AnyTableColumn<'a, T, F>
where
    T: 'static,
    F: Fn(&T) -> String,
{
    pub fn new(title: &'a str, align: Align, getter: F) -> Self {
        Self {
            title,
            align,
            expand: false,
            searchable: false,
            getter,
            style: None,
        }
    }

    pub fn expand(mut self) -> Self {
        self.expand = true;
        self
    }

    pub fn searchable(mut self) -> Self {
        self.searchable = true;
        self
    }

    pub fn style<S>(mut self, style: S) -> Self
    where
        S: Fn(&T) -> CellStyle + 'static,
    {
        self.style = Some(Box::new(style));
        self
    }
}

pub trait AsTable {
    fn as_table(language: CurrentLanguage) -> AnyTable;
}

/// Maps OLAP field ids to human names, ignoring case and padding.
struct TitleResolver<'a>(HashMap<String, &'a str>);

impl<'a> TitleResolver<'a> {
    fn new(id_to_name: &'a HashMap<String, String>) -> Self {
        Self(
            id_to_name
                .iter()
                .map(|(id, name)| (normalise_id(id), name.as_str()))
                .collect(),
        )
    }

    fn title(&self, column: &str) -> String {
        if let Some(name) = self.lookup(column) {
            return name.to_string();
        }

        let split = column.find(" / ").into_iter().chain(column.find('[')).min();

        if let Some(at) = split
            && let Some(name) = self.lookup(&column[..at])
        {
            return format!("{name}{}", &column[at..]);
        }

        column.to_string()
    }

    fn lookup(&self, column: &str) -> Option<&str> {
        self.0.get(&normalise_id(column)).copied()
    }
}

fn normalise_id(id: &str) -> String {
    id.trim().to_lowercase()
}

fn column_is_numeric(rows: &[Vec<String>], kinds: &[OlapRowKind], index: usize) -> bool {
    let mut seen = false;
    for (row, _) in rows
        .iter()
        .zip(kinds)
        .filter(|(_, kind)| matches!(kind, OlapRowKind::Data))
    {
        let Some(cell) = row.get(index).map(|c| c.trim()).filter(|c| !c.is_empty()) else {
            continue;
        };

        let numeric = cell
            .as_bytes()
            .first()
            .is_some_and(|b| b.is_ascii_digit() || *b == b'-' || *b == b'+')
            && cell.parse::<f64>().is_ok();
        if !numeric {
            return false;
        }
        seen = true;
    }
    seen
}
