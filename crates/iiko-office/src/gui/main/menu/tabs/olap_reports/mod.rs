use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use gtk4::{
    Align::{self},
    Button,
    Orientation::{Horizontal, Vertical},
    glib::{self, BoxedAnyObject, object::Cast},
    prelude::*,
};
use indexmap::IndexMap;
use serde_json::Value;

use crate::gui::{
    GlobalData,
    common::{
        datepicker::DatePicker,
        drag_space::DragSpace,
        period_list::PeriodList,
        table::{AnyTable, AnyTableColumn},
        utils::spawn_workflow,
    },
    main::menu::{
        tabs::{AnyTab, build_box},
        view::MainView,
    },
    translation::{
        Line::{
            DATE_FROM, DATE_TO, OLAP_AGGREGATE_FIELDS, OLAP_COLUMN_FIELDS, OLAP_FIELDS,
            OLAP_REPORTS, OLAP_ROW_FIELDS, REFRESH,
        },
        translate,
    },
};
use iiko_api::{
    consts::{PeriodType, ReportType},
    olap::{Filter, OlapAnswer},
    olap_columns::OlapColumn,
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
        let period_list = PeriodList::new(
            gdata.language(),
            glib::clone!(
                #[weak]
                date_from,
                #[weak]
                date_to,
                move |value| {
                    date_from.set_visible(value);
                    date_to.set_visible(value);
                }
            ),
        );

        date_from.attach_to(&grid, 0, 1);
        date_to.attach_to(&grid, 1, 1);
        grid.attach(period_list.present(), 0, 0, 1, 1);
        grid.attach(&button, 0, 1, 1, 1);

        olap_box.append(&grid);

        let content = build_box(Horizontal);

        let search_and_columns = build_box(Vertical);

        let columns_table = AnyTable::new(false);
        search_and_columns.append(&columns_table.search_entry());
        search_and_columns.append(columns_table.present());

        columns_table.add_column(AnyTableColumn::new(
            translate(gdata.language(), OLAP_FIELDS),
            Align::Start,
            false,
            true,
            |p: &(String, OlapColumn)| p.1.name.clone(),
        ));

        columns_table.set_row_drag(|p: &(String, OlapColumn)| p.1.name.clone());

        let table_grid = gtk4::Grid::builder()
            .column_spacing(8)
            .row_spacing(8)
            .build();

        let report_table = AnyTable::new(true);
        let aggregation_field = DragSpace::new(
            translate(gdata.language(), OLAP_AGGREGATE_FIELDS),
            gtk4::Orientation::Horizontal,
        );
        let column_field = DragSpace::new(
            translate(gdata.language(), OLAP_COLUMN_FIELDS),
            gtk4::Orientation::Horizontal,
        );
        let row_field = DragSpace::new(
            translate(gdata.language(), OLAP_ROW_FIELDS),
            gtk4::Orientation::Vertical,
        );

        table_grid.attach(aggregation_field.present(), 1, 0, 1, 1);
        table_grid.attach(column_field.present(), 1, 1, 1, 1);
        table_grid.attach(report_table.present(), 1, 2, 1, 1);
        table_grid.attach(row_field.present(), 0, 2, 1, 1);

        content.append(&search_and_columns);
        content.append(&table_grid);

        olap_box.append(&content);

        spawn_workflow(
            gdata.clone(),
            None,
            move |session| session.olap_columns(ReportType::Sales),
            glib::clone!(
                #[weak]
                columns_table,
                move |columns| {
                    for column in columns {
                        columns_table.add_object(&BoxedAnyObject::new(column));
                    }
                }
            ),
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
            #[weak]
            row_field,
            #[weak]
            column_field,
            #[weak]
            aggregation_field,
            #[weak]
            period_list,
            #[weak]
            columns_table,
            move |button| {
                olap_callback(
                    gdata,
                    button,
                    report_table,
                    date_from,
                    date_to,
                    row_field,
                    column_field,
                    aggregation_field,
                    period_list,
                    columns_table,
                );
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
    row_field: DragSpace,
    column_field: DragSpace,
    aggregation_field: DragSpace,
    period_list: PeriodList,
    fields_table: AnyTable,
) {
    let from = date_from.get_date();
    let to = date_to.get_date();

    let fields: Vec<(String, OlapColumn)> =
        fields_table.get_items::<(String, OlapColumn)>().to_vec();

    let name_to_id: HashMap<String, String> = fields
        .iter()
        .map(|(id, col)| (col.name.clone(), id.clone()))
        .collect();

    let id_to_name: HashMap<String, String> = fields
        .iter()
        .map(|(id, col)| (id.clone(), col.name.clone()))
        .collect();

    let resolve = |space: &DragSpace| -> Vec<String> {
        let map = name_to_id.clone();
        space.items_match(move |items| {
            items
                .iter()
                .filter_map(|name| map.get(name).cloned())
                .collect()
        })
    };

    let rfield = resolve(&row_field);
    let cfield = resolve(&column_field);
    let afield = resolve(&aggregation_field);

    let period_type = period_list.get_value();

    spawn_workflow(
        gdata,
        Some(button),
        move |session| {
            let date_filter = match period_type {
                PeriodType::CUSTOM => Filter::new_date_range(from, to),
                _ => Filter::preset_date_range(period_type),
            };
            let filters = indexmap::IndexMap::from([date_filter]);
            session.olap(ReportType::Sales, false, rfield, cfield, afield, filters)
        },
        move |olap| {
            olap_table(&report_table, &olap, &id_to_name);
        },
    );
}

fn olap_table(table: &AnyTable, answer: &OlapAnswer, id_to_name: &HashMap<String, String>) {
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
        let align = if answer
            .data
            .iter()
            .filter_map(|row| row.get(key))
            .find(|v| !v.is_null())
            .is_some_and(Value::is_number)
        {
            Align::End
        } else {
            Align::Start
        };

        let title = id_to_name.get(key).cloned().unwrap_or_else(|| key.clone());
        let key_owned = key.clone();

        table.add_column(AnyTableColumn::new(
            &title,
            align,
            false,
            true,
            move |row: &IndexMap<String, Value>| {
                row.get(key_owned.as_str())
                    .map(display_value)
                    .unwrap_or_default()
            },
        ));
    }

    for row in &answer.data {
        table.add_object(&BoxedAnyObject::new(row.clone()));
    }
}

fn display_value(value: &serde_json::Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => format!("{n:.2}"),
        _ => "null".to_string(),
    }
}
