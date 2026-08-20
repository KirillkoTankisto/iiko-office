use std::any::Any;
use std::cell::RefCell;
use std::fmt::Display;
use std::iter::successors;
use std::rc::Rc;

use gtk4::Orientation::Vertical;
use gtk4::gdk::{ContentProvider, DragAction};
use gtk4::glib::{self, BoxedAnyObject, Value};
use gtk4::pango::EllipsizeMode;
use gtk4::{Align, DragSource, DropTarget, Frame, GestureClick, Label, Orientation, Widget};
use gtk4::{StringList, prelude::*};

type DragPayload = Option<Box<dyn Any>>;

pub fn drag_content<T: 'static>(value: T) -> ContentProvider {
    let payload: DragPayload = Some(Box::new(value));
    ContentProvider::for_value(&BoxedAnyObject::new(payload).to_value())
}

pub fn take_dropped<T: 'static>(value: &Value) -> Option<T> {
    let boxed = value.get::<BoxedAnyObject>().ok()?;
    let any = boxed.borrow_mut::<DragPayload>().take()?;
    match any.downcast::<T>() {
        Ok(value) => Some(*value),
        Err(any) => {
            *boxed.borrow_mut::<DragPayload>() = Some(any);
            None
        }
    }
}
#[derive(glib::Downgrade)]
pub struct DragSpace<T> {
    root: Frame,
    items: StringList,
    inner: Rc<RefCell<Vec<T>>>,
}

impl<T: Display + 'static> DragSpace<T> {
    pub fn new(title: &str, orientation: Orientation) -> Self {
        let root = Frame::new(Some(title));
        let container = gtk4::Box::builder()
            .homogeneous(false)
            .orientation(orientation)
            .width_request(60)
            .height_request(30)
            .spacing(8)
            .build();
        root.set_child(Some(&container));

        let items = StringList::default();
        let inner: Rc<RefCell<Vec<T>>> = Rc::new(RefCell::new(Vec::new()));

        let drop_target = DropTarget::new(BoxedAnyObject::static_type(), DragAction::COPY);
        drop_target.connect_drop(glib::clone!(
            #[strong]
            items,
            #[strong]
            inner,
            #[weak]
            container,
            #[upgrade_or]
            false,
            move |_, value, _, _| {
                let Ok(boxed) = value.get::<BoxedAnyObject>() else {
                    return false;
                };
                // Move the payload out; a second drop of the same drag finds `None`.
                let Some(any) = boxed.borrow_mut::<DragPayload>().take() else {
                    return false;
                };
                match any.downcast::<T>() {
                    Ok(item) => {
                        Self::add_cell(&container, &items, &inner, *item);
                        true
                    }
                    Err(any) => {
                        *boxed.borrow_mut::<DragPayload>() = Some(any);
                        false
                    }
                }
            }
        ));
        container.add_controller(drop_target);

        Self { root, items, inner }
    }

    /// Attach this to any widget to make it a source of `T`.
    pub fn drag_source(value: T) -> DragSource
    where
        T: Clone,
    {
        let source = DragSource::builder().actions(DragAction::COPY).build();
        source.connect_prepare(move |_, _, _| {
            let payload: DragPayload = Some(Box::new(value.clone()));
            Some(ContentProvider::for_value(
                &BoxedAnyObject::new(payload).to_value(),
            ))
        });
        source
    }

    fn add_cell(container: &gtk4::Box, items: &StringList, inner: &Rc<RefCell<Vec<T>>>, item: T) {
        let text = item.to_string();
        let vertical = container.orientation() == Vertical;
        let cell = Label::builder()
            .label(&text)
            .xalign(0.0)
            .valign(Align::Start)
            .halign(if vertical { Align::Fill } else { Align::Start })
            .build();
        if vertical {
            cell.set_ellipsize(EllipsizeMode::End);
            cell.set_max_width_chars(16);
        }

        let click = GestureClick::new();
        click.connect_pressed(glib::clone!(
            #[weak]
            container,
            #[weak]
            cell,
            #[strong]
            items,
            #[strong]
            inner,
            move |_, n_press, _, _| {
                let cell: &Widget = cell.upcast_ref();
                if n_press == 2
                    && let Some(index) =
                        successors(container.first_child(), |child| child.next_sibling())
                            .position(|child| child == *cell)
                {
                    items.remove(index as u32);
                    inner.borrow_mut().remove(index);
                    container.remove(cell);
                }
            }
        ));
        cell.add_controller(click);

        items.append(&text);
        inner.borrow_mut().push(item);
        container.append(&cell);
    }

    pub fn collect<U>(&self, f: impl Fn(&T) -> U) -> Vec<U> {
        self.inner.borrow().iter().map(f).collect()
    }

    pub fn present(&self) -> &Frame {
        &self.root
    }

    pub fn items(&self) -> &StringList {
        &self.items
    }

    pub fn inner(&self) -> &Rc<RefCell<Vec<T>>> {
        &self.inner
    }
}
