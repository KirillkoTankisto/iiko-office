use crate::gui::translation::{CurrentLanguage, Line, translate};
use gtk4::glib;
use std::cell::RefCell;
use std::rc::Rc;

pub trait DropDownItem: Clone + 'static {
    fn label(&self, language: CurrentLanguage) -> String;
}

struct Inner<T> {
    items: Vec<T>,
    has_sentinel: bool,
}

pub struct AnyDropDown<T: DropDownItem> {
    root: gtk4::DropDown,
    list: gtk4::StringList,
    language: CurrentLanguage,
    inner: Rc<RefCell<Inner<T>>>,
}

pub struct AnyDropDownWeak<T: DropDownItem> {
    root: glib::WeakRef<gtk4::DropDown>,
    list: glib::WeakRef<gtk4::StringList>,
    language: CurrentLanguage,
    inner: Rc<RefCell<Inner<T>>>,
}

impl<T: DropDownItem> glib::clone::Downgrade for AnyDropDown<T> {
    type Weak = AnyDropDownWeak<T>;
    fn downgrade(&self) -> Self::Weak {
        AnyDropDownWeak {
            root: glib::clone::Downgrade::downgrade(&self.root),
            list: glib::clone::Downgrade::downgrade(&self.list),
            language: self.language,
            inner: self.inner.clone(),
        }
    }
}

impl<T: DropDownItem> glib::clone::Upgrade for AnyDropDownWeak<T> {
    type Strong = AnyDropDown<T>;
    fn upgrade(&self) -> Option<Self::Strong> {
        Some(AnyDropDown {
            root: self.root.upgrade()?,
            list: self.list.upgrade()?,
            language: self.language,
            inner: self.inner.clone(),
        })
    }
}

impl<T: DropDownItem> AnyDropDown<T> {
    pub fn new(language: CurrentLanguage, width: i32, items: Vec<T>) -> Self {
        Self::build(language, width, items, None)
    }

    pub fn with_sentinel(
        language: CurrentLanguage,
        width: i32,
        items: Vec<T>,
        sentinel: Line,
    ) -> Self {
        Self::build(language, width, items, Some(sentinel))
    }

    fn build(language: CurrentLanguage, width: i32, items: Vec<T>, sentinel: Option<Line>) -> Self {
        let mut labels: Vec<String> = items.iter().map(|i| i.label(language)).collect();
        if let Some(line) = sentinel {
            labels.push(translate(language, line).to_string());
        }
        let refs: Vec<&str> = labels.iter().map(String::as_str).collect();

        let list = gtk4::StringList::new(&refs);
        let root = gtk4::DropDown::builder()
            .model(&list)
            .selected(0)
            .width_request(width)
            .build();

        Self {
            root,
            list,
            language,
            inner: Rc::new(RefCell::new(Inner {
                items,
                has_sentinel: sentinel.is_some(),
            })),
        }
    }

    pub fn selected(&self) -> Option<T> {
        self.inner
            .borrow()
            .items
            .get(self.root.selected() as usize)
            .cloned()
    }

    pub fn is_sentinel_selected(&self) -> bool {
        let inner = self.inner.borrow();
        inner.has_sentinel && self.root.selected() as usize == inner.items.len()
    }

    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.inner.borrow().items.contains(item)
    }

    pub fn push(&self, item: T) {
        let label = item.label(self.language);
        let index = {
            let mut inner = self.inner.borrow_mut();
            inner.items.push(item);
            (inner.items.len() - 1) as u32
        };
        self.list.splice(index, 0, &[label.as_str()]);
    }

    pub fn remove_selected(&self) -> Option<T> {
        let index = self.root.selected();
        let removed = {
            let mut inner = self.inner.borrow_mut();
            let i = index as usize;
            if i >= inner.items.len() {
                return None;
            }
            inner.items.remove(i)
        };
        self.list.remove(index);
        Some(removed)
    }

    pub fn connect_selected<F: Fn(Option<T>) + 'static>(&self, f: F) {
        let inner = self.inner.clone();
        self.root.connect_selected_notify(move |root| {
            let item = inner.borrow().items.get(root.selected() as usize).cloned();
            f(item);
        });
    }

    pub fn present(&self) -> &gtk4::DropDown {
        &self.root
    }
}

impl DropDownItem for String {
    fn label(&self, _: CurrentLanguage) -> String {
        self.clone()
    }
}
