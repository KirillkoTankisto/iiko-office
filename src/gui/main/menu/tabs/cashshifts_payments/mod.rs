use std::sync::Arc;

use gtk4::Align;
use gtk4::Orientation::Vertical;
use gtk4::glib::BoxedAnyObject;
use gtk4::prelude::*;

use crate::api::cashshifts_payments_list::CashShiftsPayment;
use crate::api::cashshifts_payments_list::CashShiftsPayments;
use crate::api::cashshifts_payments_list::CashShiftsPaymentsList;
use crate::gui::GlobalData;
use crate::gui::common::datetime::reformat_date;
use crate::gui::common::table::AnyTable;
use crate::gui::common::table::AnyTableColumn;
use crate::gui::main::menu::tabs::AnyTab;
use crate::gui::main::menu::view::MainView;
use crate::gui::translation::Line::DATE;
use crate::gui::translation::Line::GROUP;
use crate::gui::translation::Line::PAYMENTS;
use crate::gui::translation::Line::SUM;
use crate::gui::translation::translate;

pub struct CashShiftsPaymentsTab {
    pub id: String,
}

impl AnyTab for CashShiftsPaymentsTab {
    fn title(&self, gdata: &GlobalData) -> &str {
        translate(gdata.language(), PAYMENTS)
    }
    fn build(&self, gdata: Arc<GlobalData>, _view: &MainView) -> gtk4::Widget {
        let cashshifts_payments_box = gtk4::Box::builder()
            .orientation(Vertical)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        let table = AnyTable::new();
        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), DATE),
            Align::Start,
            false,
            |p: &CashShiftsPayment| reformat_date(&Some(p.info.creationDate.clone())),
        ));
        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), GROUP),
            Align::Center,
            false,
            |p: &CashShiftsPayment| p.info.group.to_string(),
        ));
        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), SUM),
            Align::End,
            false,
            |p: &CashShiftsPayment| p.info.sum.to_string(),
        ));

        cashshifts_payments_box.append(table.present());

        let (sender, receiver) = async_channel::bounded::<Option<CashShiftsPayments>>(1);

        let id = self.id.clone();

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
                let all_payments = connect_payments([
                    payments.cashlessRecords,
                    payments.payInRecords,
                    payments.payOutsRecords,
                ]);

                for payment in all_payments {
                    table.add_object(&BoxedAnyObject::new(payment));
                }
            }
        });

        cashshifts_payments_box.upcast()
    }
}

fn connect_payments<const N: usize>(list: [Vec<CashShiftsPayment>; N]) -> Vec<CashShiftsPayment> {
    let mut connected: Vec<CashShiftsPayment> = list.into_iter().flatten().collect();
    connected.sort_by_key(|c| c.info.creationDate.clone());
    connected
}
