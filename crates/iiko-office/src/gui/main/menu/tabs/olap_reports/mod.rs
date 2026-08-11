use std::{collections::HashMap, sync::Arc};

use gtk4::{
    Align, Button, Grid,
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
    olap::{Filter, GroupOptions, OlapRequest},
    olap_columns::OlapColumn,
};

fn column_name(column: &(String, OlapColumn)) -> String {
    column.1.name.clone()
}

fn grid() -> Grid {
    Grid::builder().column_spacing(8).row_spacing(8).build()
}

pub struct OlapReportsTab;

impl AsTable for OlapReportsTab {
    fn as_table(language: CurrentLanguage) -> AnyTable {
        let table = AnyTable::new(false);
        table.add_column(
            AnyTableColumn::new(translate(language, OLAP_FIELDS), Align::Start, column_name)
                .searchable(),
        );
        table.set_row_drag(column_name);
        table
    }
}

impl AnyTab for OlapReportsTab {
    fn title(&self, gdata: &GlobalData) -> &str {
        translate(gdata.language(), OLAP_REPORTS)
    }

    fn build(&self, gdata: Arc<GlobalData>, _view: &MainView) -> gtk4::Widget {
        let language = gdata.language();
        let olap_box = build_box(Vertical);

        let date_from_to = DateFromToPicker::new(language);
        let button = Button::with_label(translate(language, REFRESH));
        let period_list = PeriodList::new(
            language,
            glib::clone!(
                #[weak]
                date_from_to,
                move |value| date_from_to.set_visible(value)
            ),
        );

        let controls = grid();
        date_from_to.attach_to(&controls, 0, 1);
        controls.attach(period_list.present(), 0, 0, 1, 1);
        controls.attach(&button, 0, 1, 1, 1);
        olap_box.append(&controls);

        let columns_table = Self::as_table(language);
        let search_and_columns = build_box(Vertical);
        search_and_columns.append(&columns_table.search_entry());
        search_and_columns.append(columns_table.present());

        let report_table = AnyTable::new(true);
        let olap_fields = DraggableOlapFields::new(language);

        let table_grid = grid();
        table_grid.attach(olap_fields.aggregation_field.present(), 1, 0, 1, 1);
        table_grid.attach(olap_fields.column_field.present(), 1, 1, 1, 1);
        table_grid.attach(report_table.present(), 1, 2, 1, 1);
        table_grid.attach(olap_fields.row_field.present(), 0, 2, 1, 1);

        let content = build_box(Horizontal);
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
        let space = |line, orientation| DragSpace::new(translate(language, line), orientation);
        Self {
            row_field: space(OLAP_ROW_FIELDS, Vertical),
            column_field: space(OLAP_COLUMN_FIELDS, Horizontal),
            aggregation_field: space(OLAP_AGGREGATE_FIELDS, Horizontal),
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
    let period_type = period_list.get_value();

    let mut name_to_id: HashMap<String, String> = HashMap::new();
    let mut id_to_name: HashMap<String, String> = HashMap::new();
    for (id, column) in fields_table.get_items::<(String, OlapColumn)>() {
        name_to_id.insert(column.name.clone(), id.clone());
        id_to_name.insert(id, column.name);
    }

    let ids_of = |space: &DragSpace| -> Vec<String> {
        space
            .items()
            .iter()
            .filter_map(|name| name_to_id.get(name).cloned())
            .collect()
    };

    let rows = ids_of(&olap_fields.row_field);
    let cols = ids_of(&olap_fields.column_field);
    let aggregates = ids_of(&olap_fields.aggregation_field);

    let row_fields = rows.clone();
    let col_field = cols.first().cloned();
    let value_field = aggregates.first().cloned();

    spawn_workflow(
        gdata.clone(),
        Some(button),
        move |session| {
            let date_filter = match period_type {
                PeriodType::Custom => Filter::custom_date_range(from, to),
                _ => Filter::preset_date_range(period_type),
            };
            session.olap(&OlapRequest {
                report_type: ReportType::Sales,
                build_summary: false,
                group_by_row_fields: rows,
                group_by_col_fields: cols,
                aggregate_fields: aggregates,
                filters: indexmap::IndexMap::from([(
                    String::from(Filter::OPEN_DATE_FIELD),
                    date_filter,
                )]),
            })
        },
        move |olap| {
            let total = translate(gdata.language(), TOTAL);

            let (data, layout) = match col_field.as_deref().zip(value_field.as_deref()) {
                Some((col, value)) => (
                    olap.to_pivot_table(&row_fields, col, value, total),
                    OlapLayout::Pivot,
                ),
                None => (
                    olap.to_table_grouped(&row_fields, GroupOptions::grouped(total)),
                    OlapLayout::Grouped,
                ),
            };

            report_table.set_olap_table(data, &id_to_name, layout);
            report_table.add_final();
        },
    );
}
