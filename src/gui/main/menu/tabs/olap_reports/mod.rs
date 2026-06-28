use std::{collections::HashSet, sync::Arc};

use gtk4::{
    Align::{self}, Button, Orientation::{Horizontal, Vertical}, glib::{self, BoxedAnyObject, object::Cast}, prelude::*
};
use indexmap::IndexMap;
use serde_json::Value;

use crate::{
    api::{
        olap::{Filter, OlapAnswer, OlapRequest},
        olap_columns::{OlapColumn, OlapColumnRequest, ReportType},
    },
    gui::{
        GlobalData,
        common::{
            datepicker::DatePicker,
            drag_space::DragSpace,
            table::{AnyTable, AnyTableColumn},
            utils::spawn_workflow,
        },
        main::menu::{
            tabs::{AnyTab, build_box},
            view::MainView,
        },
        translation::{
            Line::{DATE_FROM, DATE_TO, OLAP_FIELDS, OLAP_REPORTS, REFRESH},
            translate,
        },
    },
};

pub struct OlapReportsTab;

impl AnyTab for OlapReportsTab {
    fn title(&self, gdata: &GlobalData) -> &str {
        translate(gdata.language(), OLAP_REPORTS)
    }

    fn build(&self, gdata: Arc<GlobalData>, _view: &MainView) -> gtk4::Widget {
        let olap_box = build_box(Vertical);

        let grid = gtk4::Grid::builder()
            .column_spacing(8)
            .row_spacing(8)
            .build();

        let date_from = DatePicker::new(translate(gdata.language(), DATE_FROM), gdata.language());
        let date_to = DatePicker::new(translate(gdata.language(), DATE_TO), gdata.language());
        let button = gtk4::Button::with_label(translate(gdata.language(), REFRESH));

        date_from.attach_to(&grid, 0);
        date_to.attach_to(&grid, 1);
        grid.attach(&button, 1, 2, 1, 1);

        olap_box.append(&grid);

        let content = build_box(Horizontal);

        let table = AnyTable::new(false);

        table.add_column(AnyTableColumn::new(
            translate(gdata.language(), OLAP_FIELDS),
            Align::Start,
            false,
            |p: &(String, OlapColumn)| p.1.name.clone(),
        ));

        table.set_row_drag(|p: &(String, OlapColumn)| p.1.name.clone());

        let table_grid = gtk4::Grid::builder()
            .column_spacing(8)
            .row_spacing(8)
            .build();

        let report_table = AnyTable::new(true);
        let aggregation_field = DragSpace::new(gtk4::Orientation::Horizontal);
        let column_field = DragSpace::new(gtk4::Orientation::Horizontal);
        let row_field = DragSpace::new(gtk4::Orientation::Vertical);

        table_grid.attach(aggregation_field.present(), 1, 0, 1, 1);
        table_grid.attach(column_field.present(), 1, 1, 1, 1);
        table_grid.attach(report_table.present(), 1, 2, 1, 1);
        table_grid.attach(row_field.present(), 0, 2, 1, 1);

        content.append(table.present());
        content.append(&table_grid);

        olap_box.append(&content);

        spawn_workflow(
            gdata.clone(),
            None,
            move |udata| {
                OlapColumnRequest::new(&udata.address, &udata.token, ReportType::SALES).run()
            },
            move |columns| {
                for column in columns {
                    table.add_object(&BoxedAnyObject::new(column));
                }
            },
        );

        button.connect_clicked(glib::clone!(
            #[weak]
            gdata,
            #[weak]
            report_table,
            #[weak]
            date_from,
            #[weak]
            date_to,
            move |button| {
                olap_callback(gdata, button, report_table, date_from, date_to);
            }
        ));

        olap_box.upcast()
    }
}

fn olap_callback(
    gdata: Arc<GlobalData>,
    button: &Button,
    report_table: AnyTable,
    date_from: DatePicker,
    date_to: DatePicker,
) {
    button.set_sensitive(false);
    let from = date_from.get_date();
    let to = date_to.get_date();

    spawn_workflow(
        gdata,
        Some(button),
        move |udata| {
            let date_filter = Filter::new_date_range(from, to);
            let filters = indexmap::IndexMap::from([date_filter]);
            OlapRequest::new(
                &udata.address,
                &udata.token,
                ReportType::SALES,
                None,
                vec![String::from("DishCategory")],
                vec![],
                vec![
                    String::from("GuestNum"),
                    String::from("DishSumInt"),
                    String::from("DishDiscountSumInt"),
                    String::from("UniqOrderId"),
                ],
                filters,
            )
            .run()
        },
        move |olap| {
            olap_table(&report_table, &olap);
        },
    );
}

fn olap_table(table: &AnyTable, answer: &OlapAnswer) {
    table.clear_table();
    table.remove_columns();

    let mut columns: Vec<String> = Vec::new();
    let mut seen = HashSet::new();

    for row in &answer.data {
        for key in row.keys() {
            if seen.insert(key.clone()) {
                columns.push(key.clone());
            }
        }
    }

    for key in &columns {
        let align = {
            if answer
                .data
                .iter()
                .filter_map(|row| row.get(key))
                .find(|v| !v.is_null())
                .is_some_and(Value::is_number)
            {
                Align::End
            } else {
                Align::Start
            }
        };

        let key_owned = key.clone();

        table.add_column(AnyTableColumn::new(
            key,
            align,
            false,
            move |row: &IndexMap<String, Value>| {
                row.get(key_owned.as_str())
                    .map(|v| v.to_string())
                    .unwrap_or_default()
            },
        ));
    }

    for row in &answer.data {
        table.add_object(&BoxedAnyObject::new(row.clone()));
    }
}
