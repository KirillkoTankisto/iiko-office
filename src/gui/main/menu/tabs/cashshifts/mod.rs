use std::sync::Arc;

use gtk4::Align::{self, Fill};
use gtk4::{
    Box, Button, ColumnView, Notebook, Orientation::Vertical, ScrolledWindow, SingleSelection,
    gio::ListStore, glib::BoxedAnyObject,
};

use gtk4::prelude::*;

use crate::gui::main::menu::tabs::cashshifts_payments::create_cashshifts_payments;
use crate::gui::translation::Line::{
    ACCEPT_DATE, CLOSE_DATE, OPEN_DATE, REFRESH, SALES_CARD, SALES_CASH, SALES_CREDIT, SALES_SUM,
    SHIFT_NUMBER,
};
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

    let date_from = DatePicker::new(translate(gdata.language, DATE_FROM), gdata.language());
    let date_to = DatePicker::new(translate(gdata.language, DATE_TO), gdata.language());
    let refresh_button = Button::with_label(translate(gdata.language, REFRESH));
    let (table, store) = build_empty_table(gdata.clone(), view);
    let scrolled_window = ScrolledWindow::builder()
        .child(&table)
        .halign(Align::Fill)
        .valign(Align::Fill)
        .hexpand(true)
        .vexpand(true)
        .build();

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
            translate(gdata.language, CASH_SHIFTS),
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

    let (sender, receiver) = async_channel::bounded::<Option<CashShifts>>(1);

    std::thread::spawn(move || {
        if let Some((address, token)) = {
            if let Ok(udata) = gdata.user_data.lock() {
                Some((udata.address.clone(), udata.token.clone()))
            } else {
                None
            }
        } {
            let cashshifts_list =
                CashShiftsList::new(address, token, date_from, date_to, "ANY".into());
            let _ = sender.send_blocking(cashshifts_list.run().ok());
        } else {
            let _ = sender.send_blocking(None);
        }
    });

    let store = store.clone();
    let button = button.clone();
    gtk4::glib::spawn_future_local(async move {
        if let Ok(received) = receiver.recv().await
            && let Some(cashshifts) = received
        {
            store.remove_all();
            for s in cashshifts {
                store.append(&BoxedAnyObject::new(s));
            }
        }

        button.set_sensitive(true);
    });
}

fn build_empty_table(gdata: Arc<GlobalData>, view: &Notebook) -> (ColumnView, ListStore) {
    let store = ListStore::new::<BoxedAnyObject>();
    let selection = SingleSelection::new(Some(store.clone()));
    let column_view = ColumnView::new(Some(selection));
    column_view.set_hexpand(true);
    column_view.set_halign(Fill);

    add_col(
        &column_view,
        translate(gdata.language, OPEN_DATE),
        Align::Start,
        |s: &CashShift| reformat_date(&Some(s.openDate.clone())),
    );
    add_col(
        &column_view,
        translate(gdata.language, CLOSE_DATE),
        Align::Start,
        |s: &CashShift| reformat_date(&s.closeDate.clone()),
    );
    add_col(
        &column_view,
        translate(gdata.language, ACCEPT_DATE),
        Align::Start,
        |s: &CashShift| reformat_date(&s.acceptDate.clone()),
    );
    add_col(
        &column_view,
        translate(gdata.language, SALES_SUM),
        Align::End,
        |s: &CashShift| (s.salesCash + s.salesCredit + s.salesCard).to_string(),
    );
    add_col(
        &column_view,
        translate(gdata.language, SALES_CARD),
        Align::End,
        |s: &CashShift| s.salesCard.to_string(),
    );
    add_col(
        &column_view,
        translate(gdata.language, SALES_CASH),
        Align::End,
        |s: &CashShift| s.salesCash.to_string(),
    );
    add_col(
        &column_view,
        translate(gdata.language, SALES_CREDIT),
        Align::End,
        |s: &CashShift| s.salesCredit.to_string(),
    );
    add_col(
        &column_view,
        translate(gdata.language, SHIFT_NUMBER),
        Align::End,
        |s: &CashShift| s.sessionNumber.to_string(),
    );

    let view = view.clone();
    column_view.connect_activate(move |cview, row| {
        let model = cview
            .model()
            .expect("ColumnView has no model (Cash Shifts)");
        let item = model.item(row).expect("No item at that position");
        let object = item.downcast_ref::<BoxedAnyObject>().unwrap();
        let id = object.borrow::<CashShift>().id.clone();
        create_cashshifts_payments(gdata.clone(), &view, id);
    });

    (column_view, store)
}
