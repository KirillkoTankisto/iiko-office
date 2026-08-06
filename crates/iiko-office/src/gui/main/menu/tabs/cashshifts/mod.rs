use std::sync::Arc;

use gtk4::Align::{self};
use gtk4::Orientation::Vertical;
use gtk4::{Button, glib::BoxedAnyObject};

use gtk4::glib;
use gtk4::prelude::*;
use iiko_api::cashshifts_list::{CashShift, SessionStatus};

use crate::gui::common::datepicker::DateFromToPicker;
use crate::gui::common::table::{AnyTable, AnyTableColumn, AsTable};
use crate::gui::common::utils::spawn_workflow;
use crate::gui::main::menu::tabs::cashshifts_payments::CashShiftsPaymentsTab;
use crate::gui::main::menu::tabs::{AnyTab, build_box};
use crate::gui::main::menu::view::MainView;
use crate::gui::translation::CurrentLanguage;
use crate::gui::translation::Line::{
    ACCEPT_DATE, CLOSE_DATE, OPEN_DATE, REFRESH, SALES_CARD, SALES_CASH, SALES_CREDIT, SALES_SUM,
    SHIFT_NUMBER,
};
use crate::gui::{
    GlobalData,
    common::datetime::reformat_date,
    translation::{Line::CASH_SHIFTS, translate},
};

pub struct CashShiftsTab;

impl AsTable for CashShiftsTab {
    fn as_table(language: CurrentLanguage) -> AnyTable {
        let table = AnyTable::new(true);
        table.add_column(AnyTableColumn::new(
            translate(language, OPEN_DATE),
            Align::Start,
            |s: &CashShift| reformat_date(Some(&s.open_date)),
        ));
        table.add_column(AnyTableColumn::new(
            translate(language, CLOSE_DATE),
            Align::Start,
            |s: &CashShift| reformat_date(s.close_date.as_deref()),
        ));
        table.add_column(AnyTableColumn::new(
            translate(language, ACCEPT_DATE),
            Align::Start,
            |s: &CashShift| reformat_date(s.accept_date.as_deref()),
        ));
        table.add_column(AnyTableColumn::new(
            translate(language, SALES_SUM),
            Align::End,
            |s: &CashShift| (s.sales_cash + s.sales_card + s.sales_credit).to_string(),
        ));
        table.add_column(AnyTableColumn::new(
            translate(language, SALES_CARD),
            Align::End,
            |s: &CashShift| s.sales_card.to_string(),
        ));
        table.add_column(AnyTableColumn::new(
            translate(language, SALES_CASH),
            Align::End,
            |s: &CashShift| s.sales_cash.to_string(),
        ));
        table.add_column(AnyTableColumn::new(
            translate(language, SALES_CREDIT),
            Align::End,
            |s: &CashShift| s.sales_credit.to_string(),
        ));
        table.add_column(
            AnyTableColumn::new(
                translate(language, SHIFT_NUMBER),
                Align::End,
                |s: &CashShift| s.session_number.to_string(),
            )
            .expand(),
        );

        table
    }
}

impl AnyTab for CashShiftsTab {
    fn title(&self, gdata: &GlobalData) -> &str {
        translate(gdata.language(), CASH_SHIFTS)
    }

    fn build(&self, gdata: Arc<GlobalData>, view: &MainView) -> gtk4::Widget {
        let view = view.clone();

        let cashshifts_box = build_box(Vertical);

        let grid = gtk4::Grid::builder()
            .column_spacing(8)
            .row_spacing(8)
            .build();

        let date_from_to = DateFromToPicker::new(gdata.language());
        date_from_to.attach_to(&grid, 0, 0);

        let refresh_button = Button::with_label(translate(gdata.language(), REFRESH));
        grid.attach(&refresh_button, 1, 2, 1, 1);

        let table = Self::as_table(gdata.language());

        table.connect(glib::clone!(
            #[weak]
            gdata,
            #[strong]
            view,
            move |column_view, row| {
                let model = column_view
                    .model()
                    .expect("Couldn't get the model (Cash Shifts)");
                let item = model
                    .item(row)
                    .expect("Couldn't get an item on that position (Cash Shifts)");
                let object = item.downcast_ref::<BoxedAnyObject>().unwrap();
                let id = object.borrow::<CashShift>().id.clone();

                view.add_tab(&CashShiftsPaymentsTab { id }, gdata, None);
            }
        ));

        cashshifts_box.append(&grid);
        cashshifts_box.append(table.present());

        refresh_button.connect_clicked(glib::clone!(
            #[weak]
            gdata,
            #[weak]
            table,
            #[weak]
            date_from_to,
            move |button| {
                cashshifts_callback(gdata, button, table, date_from_to);
            }
        ));

        cashshifts_box.upcast()
    }
}

fn cashshifts_callback(
    gdata: Arc<GlobalData>,
    button: &Button,
    table: AnyTable,
    date_from_to: DateFromToPicker,
) {
    let (from, to) = date_from_to.get_date();
    spawn_workflow(
        gdata,
        Some(button),
        move |session| session.cashshifts_list(&from, &to, SessionStatus::Any),
        move |shifts| {
            table.clear_table();
            for shift in shifts {
                table.add_object(&BoxedAnyObject::new(shift));
            }
        },
    );
}
