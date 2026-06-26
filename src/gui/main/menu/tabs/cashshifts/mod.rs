use std::sync::Arc;

use gtk4::Align::{self};
use gtk4::{Button, glib::BoxedAnyObject};

use gtk4::glib;
use gtk4::prelude::*;

use crate::gui::common::table::{AnyTable, AnyTableColumn};
use crate::gui::main::menu::tabs::cashshifts_payments::CashShiftsPaymentsTab;
use crate::gui::main::menu::tabs::{AnyTab, build_box, open_tab};
use crate::gui::main::menu::view::MainView;
use crate::gui::translation::Line::{
    ACCEPT_DATE, CLOSE_DATE, OPEN_DATE, REFRESH, SALES_CARD, SALES_CASH, SALES_CREDIT, SALES_SUM,
    SHIFT_NUMBER,
};
use crate::{
    api::cashshifts_list::{CashShift, CashShifts, CashShiftsList},
    gui::{
        GlobalData,
        common::{datepicker::DatePicker, datetime::reformat_date},
        translation::{
            Line::{CASH_SHIFTS, DATE_FROM, DATE_TO},
            translate,
        },
    },
};

pub struct CashShiftsTab;

impl AnyTab for CashShiftsTab {
    fn title(&self, gdata: &GlobalData) -> &str {
        translate(gdata.language(), CASH_SHIFTS)
    }

    fn build(&self, gdata: Arc<GlobalData>, view: &MainView) -> gtk4::Widget {
        let view = view.clone();

        let cashshifts_box = build_box();

        let grid = gtk4::Grid::builder()
            .column_spacing(8)
            .row_spacing(8)
            .build();

        let date_from = DatePicker::new(translate(gdata.language(), DATE_FROM), gdata.language());
        let date_to = DatePicker::new(translate(gdata.language(), DATE_TO), gdata.language());
        let refresh_button = Button::with_label(translate(gdata.language(), REFRESH));

        date_from.attach_to(&grid, 0);
        date_to.attach_to(&grid, 1);
        grid.attach(&refresh_button, 1, 2, 1, 1);

        let table = AnyTable::new();
        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), OPEN_DATE),
            Align::Start,
            false,
            |s: &CashShift| reformat_date(&Some(s.openDate.clone())),
        ));
        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), CLOSE_DATE),
            Align::Start,
            false,
            |s: &CashShift| reformat_date(&s.closeDate),
        ));
        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), ACCEPT_DATE),
            Align::Start,
            false,
            |s: &CashShift| reformat_date(&s.acceptDate),
        ));
        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), SALES_SUM),
            Align::End,
            false,
            |s: &CashShift| (s.salesCash + s.salesCard + s.salesCredit).to_string(),
        ));
        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), SALES_CARD),
            Align::End,
            false,
            |s: &CashShift| s.salesCard.to_string(),
        ));
        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), SALES_CASH),
            Align::End,
            false,
            |s: &CashShift| s.salesCash.to_string(),
        ));
        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), SALES_CREDIT),
            Align::End,
            false,
            |s: &CashShift| s.salesCredit.to_string(),
        ));
        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), SHIFT_NUMBER),
            Align::End,
            true,
            |s: &CashShift| s.sessionNumber.to_string(),
        ));

        table.connect(glib::clone!(
            #[strong]
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

                open_tab(&CashShiftsPaymentsTab { id }, gdata.clone(), &view, None);
            }
        ));

        cashshifts_box.append(&grid);
        cashshifts_box.append(table.present());

        refresh_button.connect_clicked(glib::clone!(
            #[strong]
            gdata,
            #[strong]
            table,
            move |button| {
                cashshifts_callback(
                    gdata.clone(),
                    button,
                    table.clone(),
                    date_from.clone(),
                    date_to.clone(),
                );
            }
        ));

        cashshifts_box.upcast()
    }
}

fn cashshifts_callback(
    gdata: Arc<GlobalData>,
    button: &Button,
    table: AnyTable,
    date_from: DatePicker,
    date_to: DatePicker,
) {
    let date_from = date_from.get_date();
    let date_to = date_to.get_date();
    button.set_sensitive(false);

    let (sender, receiver) = async_channel::bounded::<Result<CashShifts, String>>(1);

    std::thread::spawn(move || {
        if let Some(udata) = gdata.get_credentials() {
            let cashshifts_list =
                CashShiftsList::new(udata.address, udata.token, date_from, date_to, "ANY");
            let _ = sender.send_blocking(cashshifts_list.run());
        } else {
            let _ = sender.send_blocking(Err("Couldn't get udata (Cash Shifts)".to_string()));
        }
    });

    gtk4::glib::spawn_future_local(glib::clone!(
        #[weak]
        button,
        async move {
            if let Ok(received) = receiver.recv().await {
                match received {
                    Ok(cashshifts) => {
                        table.clear_table();
                        for shift in cashshifts {
                            table.add_object(&BoxedAnyObject::new(shift));
                        }
                    }
                    Err(s) => eprintln!("{s}"),
                }
            }

            button.set_sensitive(true);
        }
    ));
}
