use std::sync::Arc;

use gtk4::{Align, Orientation::Vertical, glib::BoxedAnyObject, prelude::*};
use iiko_api::cashshifts_payments_list::CashShiftsPayment;

use crate::gui::{
    GlobalData,
    common::{
        datetime::reformat_date,
        table::{AnyTable, AsTable, ColumnSpec},
        utils::spawn_workflow,
    },
    main::menu::{
        tabs::{AnyTab, build_box},
        view::MainView,
    },
    translation::{
        CurrentLanguage,
        Line::{DATE, GROUP, PAYMENTS, SUM},
        translate,
    },
};

pub struct CashShiftsPaymentsTab {
    pub id: String,
}

const COLUMNS: &[ColumnSpec<CashShiftsPayment>] = &[
    ColumnSpec::new(DATE, Align::Start, |p| {
        reformat_date(Some(&p.info.creation_date))
    }),
    ColumnSpec::new(GROUP, Align::Center, |p| p.info.group.to_string()),
    ColumnSpec::new(SUM, Align::End, |p| p.info.sum.to_string()),
];

impl AsTable for CashShiftsPaymentsTab {
    fn as_table(language: CurrentLanguage) -> AnyTable {
        let table = AnyTable::new(true);
        table.add_columns(language, COLUMNS);
        table.add_final();
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
