use std::sync::Arc;

use gtk4::{Align, glib, prelude::*};
use iiko_api::cashshifts_payments_list::CashShiftsPayment;

use crate::gui::{
    GlobalData,
    common::{
        anybox::AnyBox,
        datetime::reformat_date,
        table::{AnyTable, ColumnSpec, GetTable},
        utils::spawn_workflow,
    },
    main::menu::{tabs::AnyTab, view::MainView},
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

impl GetTable<CashShiftsPayment> for CashShiftsPaymentsTab {
    fn get_table(language: CurrentLanguage) -> AnyTable<CashShiftsPayment> {
        let table: AnyTable<CashShiftsPayment> = AnyTable::new(true);
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
        let table = Self::get_table(gdata.language());

        let abox = AnyBox::vertical()
            .margin(8)
            .add_widgets([table.present().upcast_ref()])
            .consume();

        let id = &self.id;

        spawn_workflow(
            gdata,
            None,
            glib::clone!(
                #[strong]
                id,
                move |session| session.cashshifts_payments_list(&id, false)
            ),
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
                    table.add_object(payment);
                }
            },
        );

        abox.upcast()
    }
}
