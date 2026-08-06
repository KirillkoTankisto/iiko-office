use std::sync::Arc;

use gtk4::Align;
use gtk4::Orientation::Vertical;
use gtk4::glib::BoxedAnyObject;
use gtk4::prelude::*;

use crate::gui::GlobalData;
use crate::gui::common::datetime::reformat_date;
use crate::gui::common::table::AnyTable;
use crate::gui::common::table::AnyTableColumn;
use crate::gui::common::table::AsTable;
use crate::gui::common::utils::spawn_workflow;
use crate::gui::main::menu::tabs::AnyTab;
use crate::gui::main::menu::tabs::build_box;
use crate::gui::main::menu::view::MainView;
use crate::gui::translation::Line::DATE;
use crate::gui::translation::Line::GROUP;
use crate::gui::translation::Line::PAYMENTS;
use crate::gui::translation::Line::SUM;
use crate::gui::translation::translate;
use iiko_api::cashshifts_payments_list::CashShiftsPayment;

pub struct CashShiftsPaymentsTab {
    pub id: String,
}

impl AsTable for CashShiftsPaymentsTab {
    fn as_table(language: crate::gui::translation::CurrentLanguage) -> AnyTable {
        let table = AnyTable::new(true);
        table.add_column(AnyTableColumn::new(
            translate(language, DATE),
            Align::Start,
            |p: &CashShiftsPayment| reformat_date(Some(&p.info.creation_date)),
        ));
        table.add_column(AnyTableColumn::new(
            translate(language, GROUP),
            Align::Center,
            |p: &CashShiftsPayment| p.info.group.to_string(),
        ));
        table.add_column(AnyTableColumn::new(
            translate(language, SUM),
            Align::End,
            |p: &CashShiftsPayment| p.info.sum.to_string(),
        ));

        table
    }
}

impl AnyTab for CashShiftsPaymentsTab {
    fn title(&self, gdata: &GlobalData) -> &str {
        translate(gdata.language(), PAYMENTS)
    }

    fn build(&self, gdata: Arc<GlobalData>, _view: &MainView) -> gtk4::Widget {
        let cashshifts_payments_box = build_box(Vertical);

        let table = Self::as_table(gdata.language());

        cashshifts_payments_box.append(table.present());

        let id = self.id.clone();

        spawn_workflow(
            gdata,
            None,
            move |session| session.cashshifts_payments_list(&id, false),
            move |payments| {
                let mut all_payments: Vec<CashShiftsPayment> = [
                    payments.cashless_records,
                    payments.pay_in_records,
                    payments.pay_outs_records,
                ]
                .into_iter()
                .flatten()
                .collect();
                all_payments.sort_by(|a, b| a.info.creation_date.cmp(&b.info.creation_date));

                for payment in all_payments {
                    table.add_object(&BoxedAnyObject::new(payment));
                }
            },
        );

        cashshifts_payments_box.upcast()
    }
}
