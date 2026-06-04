use std::sync::Arc;

use gtk4::{
    Box, Button, ColumnView, Notebook, Orientation::Vertical, ScrolledWindow, SingleSelection, gio::ListStore, glib::BoxedAnyObject
};

use gtk4::prelude::*;

use crate::{
    api::cashshifts_list::{CashShift, CashShifts, CashShiftsList},
    gui::{
        GlobalData,
        common::{datepicker::DatePicker, datetime::reformat_date, table::add_col},
        main::menu::tabs::add_tab,
        translation::{
            Line::{CASH_SHIFTS, DATE_FROM, DATE_TO},
            translate,
        },
    },
};

pub fn create_cashshifts(gdata: Arc<GlobalData>, view: &Notebook) {
    let cashshifts_box = Box::builder()
        .orientation(Vertical)
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let grid = gtk4::Grid::builder()
        .column_spacing(8)
        .row_spacing(8)
        .build();

    let date_from = DatePicker::new(translate(gdata.language.clone(), DATE_FROM), &gdata.language);
    let date_to = DatePicker::new(translate(gdata.language.clone(), DATE_TO), &gdata.language);
    let refresh_button = Button::with_label("Refresh");
    let (table, store) = build_empty_table();
    let scrolled_window = ScrolledWindow::builder().child(&table).hexpand(true).vexpand(true).build();

    date_from.attach_to(&grid, 0);
    date_to.attach_to(&grid, 1);
    grid.attach(&refresh_button, 1, 2, 1, 1);

    cashshifts_box.append(&grid);
    cashshifts_box.append(&scrolled_window);

    let gdata_cb = gdata.clone();
    refresh_button.connect_clicked(move |button| {
        cashshifts_callback(
            gdata_cb.clone(),
            button,
            &store,
            date_from.clone(),
            date_to.clone(),
        );
    });

    view.append_page(
        &cashshifts_box,
        Some(&add_tab(
            view,
            &cashshifts_box,
            translate(gdata.language.clone(), CASH_SHIFTS),
        )),
    );
}

fn cashshifts_callback(
    gdata: Arc<GlobalData>,
    button: &Button,
    store: &ListStore,
    date_from: DatePicker,
    date_to: DatePicker,
) {
    let date_from = date_from.get_date();
    let date_to = date_to.get_date();
    button.set_sensitive(false);

    let (sender, receiver) = async_channel::bounded::<CashShifts>(1);

    std::thread::spawn(move || {
        if let Ok(user_data) = gdata.user_data.lock() {
            let cashshifts_list = CashShiftsList::new(
                user_data.address.clone(),
                user_data.token.clone(),
                date_from,
                date_to,
                "ANY".into(),
            );
            let result = cashshifts_list.run();
            if let Ok(cashshifts) = result {
                let _ = sender.send_blocking(cashshifts);
            }
            else {
                let _ = sender.send_blocking(Vec::default());
            }

        } else {
            let _ = sender.send_blocking(Vec::default());
        }


    });

    let store = store.clone();
    let button = button.clone();
    gtk4::glib::spawn_future_local(async move {
        if let Ok(cashshifts) = receiver.recv().await {
            store.remove_all();
            for s in cashshifts {
                store.append(&BoxedAnyObject::new(s));
            }
        }

        button.set_sensitive(true);
    });
}

fn build_empty_table() -> (ColumnView, ListStore) {
    let store = ListStore::new::<BoxedAnyObject>();
    let selection = SingleSelection::new(Some(store.clone()));
    let column_view = ColumnView::new(Some(selection));
    column_view.set_hexpand(true);

    add_col(&column_view, "Open Date", |s: &CashShift| reformat_date(&Some(s.openDate.clone())));
    add_col(&column_view, "Close Date", |s: &CashShift| {
        reformat_date(&s.closeDate.clone())
    });
    add_col(&column_view, "Accept Date", |s: &CashShift| {
        reformat_date(&s.acceptDate.clone())
    });
    add_col(&column_view, "Sales Summary", |s: &CashShift| {
        (s.salesCash + s.salesCredit + s.salesCard).to_string()
    });
    add_col(&column_view, "Sales Card", |s: &CashShift| {
        s.salesCard.to_string()
    });
    add_col(&column_view, "Sales Cash", |s: &CashShift| {
        s.salesCash.to_string()
    });
    add_col(&column_view, "Sales Credit", |s: &CashShift| {
        s.salesCredit.to_string()
    });
    add_col(&column_view, "Session Number", |s: &CashShift| {
        s.sessionNumber.to_string()
    });

    (column_view, store)
}
