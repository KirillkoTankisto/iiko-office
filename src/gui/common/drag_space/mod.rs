use std::{cell::RefCell, rc::Rc};

use gtk4::Orientation::Vertical;
use gtk4::prelude::*;
use gtk4::{DropTarget, Orientation, gdk::DragAction, glib::types::Type};

pub struct DragSpace {
    root: gtk4::Frame,
    items: Rc<RefCell<Vec<String>>>,
}

use gtk4::{GestureClick, glib};

impl DragSpace {
    pub fn new(orientation: Orientation) -> Self {
        let root = gtk4::Frame::new(Some("olap"));
        let container = gtk4::Box::builder()
            .homogeneous(false)
            .orientation(orientation)
            .width_request(60)
            .height_request(30)
            .spacing(8)
            .build();

        root.set_child(Some(&container));

        let items = Rc::new(RefCell::new(Vec::new()));

        let drop_target = DropTarget::new(Type::STRING, DragAction::COPY);
        drop_target.connect_drop(glib::clone!(
            #[strong]
            items,
            #[weak]
            container,
            #[upgrade_or]
            false,
            move |_, value, _, _| match value.get::<String>() {
                Ok(text) => {
                    Self::add_cell(&container, &items, text);
                    true
                }
                Err(_) => false,
            }
        ));
        container.add_controller(drop_target);

        Self { root, items }
    }

    fn add_cell(container: &gtk4::Box, items: &Rc<RefCell<Vec<String>>>, text: String) {
        let cell = gtk4::Label::builder()
            .label(&text)
            .xalign(0.0)
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::Start)
            .build();

        if container.orientation() == Vertical {
            cell.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            cell.set_max_width_chars(16);
            cell.set_halign(gtk4::Align::Fill);
        }

        let click = GestureClick::new();
        click.connect_pressed(glib::clone!(
            #[weak]
            container,
            #[weak]
            cell,
            #[strong]
            items,
            move |_, n_press, _, _| {
                if n_press == 2
                    && let Some(idx) = Self::child_index(&container, &cell)
                {
                    items.borrow_mut().remove(idx);
                    container.remove(&cell);
                }
            }
        ));
        cell.add_controller(click);

        items.borrow_mut().push(text);
        container.append(&cell);
    }

    fn child_index(container: &gtk4::Box, target: &impl IsA<gtk4::Widget>) -> Option<usize> {
        let target = target.as_ref();
        let mut child = container.first_child();
        let mut i = 0;
        while let Some(c) = child {
            if c == *target {
                return Some(i);
            }
            child = c.next_sibling();
            i += 1;
        }
        None
    }

    pub fn present(&self) -> &gtk4::Frame {
        &self.root
    }

    pub fn items(&self) -> Vec<String> {
        self.items.borrow().clone()
    }
}
