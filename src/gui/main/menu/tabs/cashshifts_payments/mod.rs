use std::sync::Arc;

use gtk4::Align;
use gtk4::ColumnView;
use gtk4::Notebook;
use gtk4::Orientation::Vertical;
use gtk4::ScrolledWindow;
use gtk4::SingleSelection;
use gtk4::gio::ListStore;
use gtk4::glib::BoxedAnyObject;
use gtk4::prelude::*;

use crate::api::cashshifts_payments_list::CashShiftsPayment;
use crate::api::cashshifts_payments_list::CashShiftsPayments;
use crate::api::cashshifts_payments_list::CashShiftsPaymentsList;
use crate::gui::GlobalData;
use crate::gui::common::datetime::reformat_date;
use crate::gui::common::table::add_col;
use crate::gui::main::menu::tabs::add_tab;
use crate::gui::translation::CurrentLanguage;
use crate::gui::translation::Line::DATE;
use crate::gui::translation::Line::GROUP;
use crate::gui::translation::Line::PAYMENTS;
use crate::gui::translation::Line::SUM;
use crate::gui::translation::translate;

pub fn create_cashshifts_payments(gdata: Arc<GlobalData>, view: &Notebook, id: String) {
    let cashshifts_payments_box = gtk4::Box::builder()
        .orientation(Vertical)
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let (table, store) = build_empty_table(gdata.language);

    let scrolled_window = ScrolledWindow::builder()
        .child(&table)
        .hexpand(true)
        .vexpand(true)
        .build();

    cashshifts_payments_box.append(&scrolled_window);
    let tab = add_tab(
        view,
        &cashshifts_payments_box,
        translate(gdata.language, PAYMENTS),
    );
    view.append_page(&cashshifts_payments_box, Some(&tab));

    let (sender, receiver) = async_channel::bounded::<Option<CashShiftsPayments>>(1);

    std::thread::spawn(move || {
        if let Some((address, token)) = {
            if let Ok(udata) = gdata.user_data.lock() {
                Some((udata.address.clone(), udata.token.clone()))
            } else {
                None
            }
        } {
            let cashshifts_payments = CashShiftsPaymentsList::new(address, token, id);
            let _ = sender.send_blocking(cashshifts_payments.run().ok());
        } else {
            eprintln!("Cannot get data from gdata");
            let _ = sender.send_blocking(None);
        }
    });

    gtk4::glib::spawn_future_local(async move {
        if let Ok(received) = receiver.recv().await
            && let Some(payments) = received
        {
            for payment in payments.cashlessRecords {
                store.append(&BoxedAnyObject::new(payment));
            }
        }
    });
}

fn build_empty_table(language: CurrentLanguage) -> (ColumnView, ListStore) {
    let store = ListStore::new::<BoxedAnyObject>();
    let selection = SingleSelection::new(Some(store.clone()));
    let column_view = ColumnView::new(Some(selection));
    column_view.set_hexpand(true);

    add_col(
        &column_view,
        translate(language, DATE),
        Align::Start,
        |p: &CashShiftsPayment| reformat_date(&Some(p.info.creationDate.clone())),
    );
    add_col(
        &column_view,
        translate(language, GROUP),
        Align::Center,
        |p: &CashShiftsPayment| p.info.group.to_string(),
    );
    add_col(
        &column_view,
        translate(language, SUM),
        Align::End,
        |p: &CashShiftsPayment| p.info.sum.to_string(),
    );

    (column_view, store)
}
