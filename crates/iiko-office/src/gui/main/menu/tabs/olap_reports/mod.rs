use std::{collections::HashMap, sync::Arc};

use gtk4::{
    Align::{self},
    Button,
    Orientation::{Horizontal, Vertical},
    glib::{self, BoxedAnyObject, object::Cast},
    prelude::*,
};

use crate::gui::{
    GlobalData,
    common::{
        datepicker::DateFromToPicker,
        drag_space::DragSpace,
        period_list::PeriodList,
        table::{AnyTable, AnyTableColumn, AsTable},
        utils::spawn_workflow,
    },
    main::menu::{
        tabs::{AnyTab, build_box},
        view::MainView,
    },
    translation::{
        CurrentLanguage,
        Line::{
            OLAP_AGGREGATE_FIELDS, OLAP_COLUMN_FIELDS, OLAP_FIELDS, OLAP_REPORTS, OLAP_ROW_FIELDS,
            REFRESH,
        },
        translate,
    },
};
use iiko_api::{
    consts::{PeriodType, ReportType},
    olap::{Filter, OlapAnswer, OlapTable},
    olap_columns::OlapColumn,
};

pub struct OlapReportsTab;

impl AsTable for OlapReportsTab {
    fn as_table(language: crate::gui::translation::CurrentLanguage) -> AnyTable {
        let table = AnyTable::new(false);
        table.add_column(AnyTableColumn::new(
            translate(language, OLAP_FIELDS),
            Align::Start,
            false,
            true,
            |p: &(String, OlapColumn)| p.1.name.clone(),
        ));

        table.set_row_drag(|p: &(String, OlapColumn)| p.1.name.clone());

        table
    }
}

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

        let date_from_to = DateFromToPicker::new(gdata.language());

        let button = gtk4::Button::with_label(translate(gdata.language(), REFRESH));
        let period_list = PeriodList::new(
            gdata.language(),
            glib::clone!(
                #[weak]
                date_from_to,
                move |value| {
                    date_from_to.set_visible(value);
                }
            ),
        );

        date_from_to.attach_to(&grid, 0, 1);
        grid.attach(period_list.present(), 0, 0, 1, 1);
        grid.attach(&button, 0, 1, 1, 1);

        olap_box.append(&grid);

        let content = build_box(Horizontal);

        let search_and_columns = build_box(Vertical);

        let columns_table = Self::as_table(gdata.language());
        search_and_columns.append(&columns_table.search_entry());
        search_and_columns.append(columns_table.present());

        let table_grid = gtk4::Grid::builder()
            .column_spacing(8)
            .row_spacing(8)
            .build();

        let report_table = AnyTable::new(true);

        let olap_fields = DraggableOlapFields::new(gdata.language());

        table_grid.attach(olap_fields.aggregation_field.present(), 1, 0, 1, 1);
        table_grid.attach(olap_fields.column_field.present(), 1, 1, 1, 1);
        table_grid.attach(report_table.present(), 1, 2, 1, 1);
        table_grid.attach(olap_fields.row_field.present(), 0, 2, 1, 1);

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
            date_from_to,
            #[weak]
            olap_fields,
            #[weak]
            period_list,
            #[weak]
            columns_table,
            move |button| {
                olap_callback(
                    gdata,
                    button,
                    report_table,
                    date_from_to,
                    olap_fields,
                    period_list,
                    columns_table,
                );
            }
        ));

        olap_box.upcast()
    }
}

#[derive(glib::Downgrade)]
pub struct DraggableOlapFields {
    row_field: DragSpace,
    column_field: DragSpace,
    aggregation_field: DragSpace,
}

impl DraggableOlapFields {
    fn new(language: CurrentLanguage) -> Self {
        Self {
            row_field: DragSpace::new(
                translate(language, OLAP_ROW_FIELDS),
                gtk4::Orientation::Vertical,
            ),
            column_field: DragSpace::new(
                translate(language, OLAP_COLUMN_FIELDS),
                gtk4::Orientation::Horizontal,
            ),
            aggregation_field: DragSpace::new(
                translate(language, OLAP_AGGREGATE_FIELDS),
                gtk4::Orientation::Horizontal,
            ),
        }
    }
}

fn olap_callback(
    gdata: Arc<GlobalData>,
    button: &Button,
    report_table: AnyTable,
    date_from_to: DateFromToPicker,
    olap_fields: DraggableOlapFields,
    period_list: PeriodList,
    fields_table: AnyTable,
) {
    let (from, to) = date_from_to.get_date();

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

    let rfield = resolve(&olap_fields.row_field);
    let cfield = resolve(&olap_fields.column_field);
    let afield = resolve(&olap_fields.aggregation_field);

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

    let OlapTable { columns, rows } = answer.to_table();

    for (index, column) in columns.iter().enumerate() {
        let align = if rows
            .iter()
            .filter_map(|row| row.get(index))
            .find(|cell| !cell.is_empty())
            .is_some_and(|cell| cell.parse::<f64>().is_ok())
        {
            Align::End
        } else {
            Align::Start
        };

        let title = id_to_name
            .get(column)
            .cloned()
            .unwrap_or_else(|| column.clone());

        table.add_column(AnyTableColumn::new(
            &title,
            align,
            false,
            true,
            move |row: &Vec<String>| row.get(index).cloned().unwrap_or_default(),
        ));
    }

    for row in rows {
        table.add_object(&BoxedAnyObject::new(row));
    }
}
