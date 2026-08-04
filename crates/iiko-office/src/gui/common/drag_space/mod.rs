use std::{cell::RefCell, iter::successors, rc::Rc};

use gtk4::Orientation::Vertical;
use gtk4::gdk::DragAction;
use gtk4::glib::{self, types::Type};
use gtk4::pango::EllipsizeMode;
use gtk4::prelude::*;
use gtk4::{Align, Box, DropTarget, Frame, GestureClick, Label, Orientation, Widget};

type Items = Rc<RefCell<Vec<String>>>;

#[derive(glib::Downgrade)]
pub struct DragSpace {
    root: Frame,
    items: Items,
}

impl DragSpace {
    pub fn new(title: &str, orientation: Orientation) -> Self {
        let root = Frame::new(Some(title));
        let container = Box::builder()
            .homogeneous(false)
            .orientation(orientation)
            .width_request(60)
            .height_request(30)
            .spacing(8)
            .build();

        root.set_child(Some(&container));

        let items: Items = Rc::new(RefCell::new(Vec::new()));

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

    fn add_cell(container: &Box, items: &Items, text: String) {
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
            move |_, n_press, _, _| {
                let cell: &Widget = cell.upcast_ref();
                if n_press == 2
                    && let Some(index) =
                        successors(container.first_child(), |child| child.next_sibling())
                            .position(|child| child == *cell)
                {
                    items.borrow_mut().remove(index);
                    container.remove(cell);
                }
            }
        ));
        cell.add_controller(click);

        items.borrow_mut().push(text);
        container.append(&cell);
    }

    pub fn present(&self) -> &Frame {
        &self.root
    }

    pub fn items(&self) -> Vec<String> {
        self.items.borrow().clone()
    }
}
