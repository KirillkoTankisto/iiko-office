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
        table::{AnyTable, AnyTableColumn, AsTable, OlapLayout},
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
            REFRESH, TOTAL,
        },
        translate,
    },
};
use iiko_api::{
    consts::{PeriodType, ReportType},
    olap::{Filter, OlapAnswer, OlapRequest},
    olap_columns::OlapColumn,
};

pub struct OlapReportsTab;

impl AsTable for OlapReportsTab {
    fn as_table(language: crate::gui::translation::CurrentLanguage) -> AnyTable {
        let table = AnyTable::new(false);
        table.add_column(
            AnyTableColumn::new(
                translate(language, OLAP_FIELDS),
                Align::Start,
                |p: &(String, OlapColumn)| p.1.name.clone(),
            )
            .searchable(),
        );

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

    let mut name_to_id: HashMap<String, String> = HashMap::new();
    let mut id_to_name: HashMap<String, String> = HashMap::new();

    for (id, column) in fields_table.get_items::<(String, OlapColumn)>() {
        name_to_id.insert(column.name.clone(), id.clone());
        id_to_name.insert(id, column.name);
    }

    let get_fields = |space: &DragSpace| -> Vec<String> {
        space
            .items()
            .iter()
            .filter_map(|name| name_to_id.get(name).cloned())
            .collect()
    };

    let rfield = get_fields(&olap_fields.row_field);
    let cfield = get_fields(&olap_fields.column_field);
    let afield = get_fields(&olap_fields.aggregation_field);

    let period_type = period_list.get_value();

    let row_fields_clone = rfield.clone();
    let col_field = cfield.first().cloned();
    let value_field = afield.first().cloned();
    spawn_workflow(
        gdata.clone(),
        Some(button),
        move |session| {
            let date_filter = match period_type {
                PeriodType::CUSTOM => Filter::custom_date_range(from, to),
                _ => Filter::preset_date_range(period_type),
            };
            let filters =
                indexmap::IndexMap::from([(String::from(Filter::OPEN_DATE_FIELD), date_filter)]);
            session.olap(&OlapRequest {
                report_type: ReportType::Sales,
                build_summary: false,
                group_by_row_fields: rfield,
                group_by_col_fields: cfield,
                aggregate_fields: afield,
                filters,
            })
        },
        move |olap| {
            olap_table(
                &report_table,
                &olap,
                &id_to_name,
                &row_fields_clone,
                col_field.as_deref(),
                value_field.as_deref(),
                translate(gdata.language(), TOTAL),
            );
        },
    );
}

fn olap_table(
    table: &AnyTable,
    answer: &OlapAnswer,
    id_to_name: &HashMap<String, String>,
    row_fields: &[String],
    col_field: Option<&str>,
    value_field: Option<&str>,
    total_label: &str,
) {
    table.clear_table();
    table.remove_columns();

    let (data, layout) = match col_field.zip(value_field) {
        Some((col, value)) => (
            answer.to_pivot_table(row_fields, col, value, total_label),
            OlapLayout::Pivot {
                key_count: row_fields.len(),
            },
        ),
        None => (answer.to_table_sorted(row_fields), OlapLayout::Flat),
    };
    table.set_olap_table(data, id_to_name, layout);
}
