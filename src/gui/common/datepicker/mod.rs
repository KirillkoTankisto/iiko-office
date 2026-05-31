use gtk4::Box;
use gtk4::Button;
use gtk4::Calendar;
use gtk4::Entry;
use gtk4::Label;
use gtk4::MenuButton;
use gtk4::Orientation::Vertical;
use gtk4::Popover;
use gtk4::prelude::*;

pub struct DatePicker {
    pub label: Label,
    pub entry: Entry,
    pub menu_button: MenuButton,
}

impl DatePicker {
    pub fn new<'a, S: Into<&'a str>>(label: S) -> Self {
        let calendar = Calendar::new();
        let close_button = Button::with_label("Close");

        let menu_box = Box::new(Vertical, 8);
        menu_box.append(&calendar);
        menu_box.append(&close_button);

        let popup = Popover::builder().child(&menu_box).build();

        let label = Label::builder().label(label.into()).xalign(0.0).build();
        let entry = Entry::builder().placeholder_text("YYYY-MM-DD").build();
        let menu_button = MenuButton::builder().popover(&popup).build();

        close_button.connect_clicked(move |_| {
            popup.popdown();
        });

        let cloned_entry = entry.clone();
        calendar.clone().connect_day_selected(move |calendar| {
            Self::set_date(calendar, &cloned_entry);
        });

        let cloned_entry = entry.clone();
        calendar.clone().connect_next_month(move |calendar| {
            Self::set_date(calendar, &cloned_entry);
        });

        let cloned_entry = entry.clone();
        calendar.clone().connect_prev_month(move |calendar| {
            Self::set_date(calendar, &cloned_entry);
        });

        let cloned_entry = entry.clone();
        calendar.clone().connect_next_year(move |calendar| {
            Self::set_date(calendar, &cloned_entry);
        });

        let cloned_entry = entry.clone();
        calendar.clone().connect_prev_year(move |calendar| {
            Self::set_date(calendar, &cloned_entry);
        });

        Self {
            label,
            entry,
            menu_button,
        }
    }

    fn set_date(calendar: &Calendar, entry: &Entry) {
        let date = calendar.date().format("%F").unwrap_or_default().to_string();
        entry.set_text(&date);
    }

    pub fn get_date(&self) -> String {
        self.entry.text().to_string()
    }

    pub fn attach_to(&self, grid: &gtk4::Grid, row: i32) {
        grid.attach(&self.label, 0, row, 1, 1);
        grid.attach(&self.entry, 1, row, 1, 1);
        grid.attach(&self.menu_button, 2, row, 1, 1);
    }
}
