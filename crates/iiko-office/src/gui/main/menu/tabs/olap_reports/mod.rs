use std::{
    collections::HashMap,
    fmt::{self, Display},
    sync::Arc,
};

use gtk4::{
    Align, Box as GtkBox, Button, Grid,
    Orientation::{Horizontal, Vertical},
    glib::{self, BoxedAnyObject, object::Cast},
    prelude::*,
};

use crate::gui::{
    GlobalData,
    common::{
        datepicker::DateFromToPicker,
        drag_space::DragSpace,
        dropdown::{AnyDropDown, DropDownItem},
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
            self, OLAP_AGGREGATE_FIELDS, OLAP_COLUMN_FIELDS, OLAP_FIELDS,
            OLAP_REPORT_TYPE_DELIVERIES, OLAP_REPORT_TYPE_SALES, OLAP_REPORT_TYPE_TRANSACTIONS,
            OLAP_REPORTS, OLAP_ROW_FIELDS, REFRESH, TOTAL,
        },
        translate,
    },
};

use iiko_api::{
    consts::{PeriodType, ReportType},
    olap::{Filter, GroupOptions, OlapRequest},
    olap_columns::OlapColumn,
};

const REPORTS: &[ReportType] = &[
    ReportType::Sales,
    ReportType::Transactions,
    ReportType::Deliveries,
];

const fn report_line(report_type: ReportType) -> Line {
    match report_type {
        ReportType::Sales => OLAP_REPORT_TYPE_SALES,
        ReportType::Transactions => OLAP_REPORT_TYPE_TRANSACTIONS,
        ReportType::Deliveries => OLAP_REPORT_TYPE_DELIVERIES,
    }
}

impl DropDownItem for ReportType {
    fn label(&self, language: CurrentLanguage) -> String {
        translate(language, report_line(*self)).to_string()
    }
}

#[derive(Clone)]
pub struct FieldRef {
    pub id: String,
    pub name: String,
}

impl Display for FieldRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

fn field_ref((id, column): &(String, OlapColumn)) -> FieldRef {
    FieldRef {
        id: id.clone(),
        name: column.name.clone(),
    }
}

fn column_name(column: &(String, OlapColumn)) -> String {
    column.1.name.clone()
}

/// get Field Id => Column Name map from AnyTable
fn field_names(fields_table: &AnyTable) -> HashMap<String, String> {
    fields_table
        .get_items::<(String, OlapColumn)>()
        .into_iter()
        .map(|(id, column)| (id, column.name))
        .collect()
}

/// Builds a simple grid
fn grid() -> Grid {
    Grid::builder().column_spacing(8).row_spacing(8).build()
}

#[derive(glib::Downgrade)]
pub struct ReportControls {
    report_type: AnyDropDown<ReportType>,
    period_list: AnyDropDown<PeriodType>,
    date_from_to: DateFromToPicker,
    refresh: Button,
}

impl ReportControls {
    fn new(language: CurrentLanguage) -> Self {
        let date_from_to = DateFromToPicker::new(language);

        // The from/to pickers are only meaningful for a custom period.
        let period_list = PeriodList::build(
            language,
            glib::clone!(
                #[weak]
                date_from_to,
                move |value| date_from_to.set_visible(value == Some(PeriodType::Custom))
            ),
        );

        Self {
            report_type: AnyDropDown::new(language, 180, REPORTS.to_vec()),
            period_list,
            date_from_to,
            refresh: Button::with_label(translate(language, REFRESH)),
        }
    }

    fn present(&self) -> Grid {
        let controls = grid();
        controls.attach(self.report_type.present(), 0, 0, 1, 1);
        self.date_from_to.attach_to(&controls, 1, 1);
        controls.attach(self.period_list.present(), 0, 1, 1, 1);
        controls.attach(&self.refresh, 0, 2, 1, 1);
        controls
    }

    fn selected_report(&self) -> ReportType {
        self.report_type.selected().unwrap_or_default()
    }

    fn date_filter(&self) -> Filter {
        match self.period_list.selected().unwrap_or_default() {
            PeriodType::Custom => {
                let (from, to) = self.date_from_to.get_date();
                Filter::custom_date_range(from, to)
            }
            period_type => Filter::preset_date_range(period_type),
        }
    }
}

#[derive(glib::Downgrade)]
pub struct DraggableOlapFields {
    row_field: DragSpace<FieldRef>,
    column_field: DragSpace<FieldRef>,
    aggregation_field: DragSpace<FieldRef>,
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

    fn selected(&self) -> SelectedFields {
        let ids = |space: &DragSpace<FieldRef>| space.collect(|field| field.id.clone());
        SelectedFields {
            rows: ids(&self.row_field),
            cols: ids(&self.column_field),
            aggregates: ids(&self.aggregation_field),
        }
    }
}

struct SelectedFields {
    rows: Vec<String>,
    cols: Vec<String>,
    aggregates: Vec<String>,
}

struct PivotSpec {
    rows: Vec<String>,
    pivot: Option<(String, String)>,
}

impl PivotSpec {
    fn new(fields: &SelectedFields) -> Self {
        Self {
            rows: fields.rows.clone(),
            pivot: fields
                .cols
                .first()
                .cloned()
                .zip(fields.aggregates.first().cloned()),
        }
    }
}

pub struct OlapReportsTab;

impl AsTable for OlapReportsTab {
    fn as_table(language: CurrentLanguage) -> AnyTable {
        let table = AnyTable::new(false);
        table.add_column(
            AnyTableColumn::new(translate(language, OLAP_FIELDS), Align::Start, column_name)
                .searchable(),
        );
        table.set_row_drag(field_ref);
        table
    }
}

impl AnyTab for OlapReportsTab {
    fn title(&self, gdata: &GlobalData) -> &str {
        translate(gdata.language(), OLAP_REPORTS)
    }

    fn build(&self, gdata: Arc<GlobalData>, _view: &MainView) -> gtk4::Widget {
        let language = gdata.language();

        let controls = ReportControls::new(language);
        let columns_table = Self::as_table(language);
        let report_table = AnyTable::new(true);
        let olap_fields = DraggableOlapFields::new(language);

        // Switching the report type replaces the list of available fields.
        controls.report_type.connect_selected(glib::clone!(
            #[weak]
            gdata,
            #[weak]
            columns_table,
            move |report_type| {
                load_columns(gdata, columns_table, report_type.unwrap_or_default());
            }
        ));

        controls.refresh.connect_clicked(glib::clone!(
            #[weak]
            gdata,
            #[weak]
            controls,
            #[weak]
            olap_fields,
            #[weak]
            columns_table,
            #[weak]
            report_table,
            move |button| {
                run_report(
                    gdata,
                    button,
                    &controls,
                    &olap_fields,
                    &columns_table,
                    report_table,
                );
            }
        ));

        let widget = layout(&controls, &columns_table, &report_table, &olap_fields);
        load_columns(gdata, columns_table, ReportType::Sales);
        widget
    }
}

fn layout(
    controls: &ReportControls,
    columns_table: &AnyTable,
    report_table: &AnyTable,
    olap_fields: &DraggableOlapFields,
) -> gtk4::Widget {
    let content = build_box(Horizontal);
    content.append(&columns_panel(columns_table));
    content.append(&pivot_grid(olap_fields, report_table));

    let olap_box = build_box(Vertical);
    olap_box.append(&controls.present());
    olap_box.append(&content);
    olap_box.upcast()
}

fn columns_panel(columns_table: &AnyTable) -> GtkBox {
    let panel = build_box(Vertical);
    panel.append(&columns_table.search_entry());
    panel.append(columns_table.present());
    panel
}

fn pivot_grid(olap_fields: &DraggableOlapFields, report_table: &AnyTable) -> Grid {
    let table_grid = grid();
    table_grid.attach(olap_fields.aggregation_field.present(), 1, 0, 1, 1);
    table_grid.attach(olap_fields.column_field.present(), 1, 1, 1, 1);
    table_grid.attach(report_table.present(), 1, 2, 1, 1);
    table_grid.attach(olap_fields.row_field.present(), 0, 2, 1, 1);
    table_grid
}

fn load_columns(gdata: Arc<GlobalData>, columns_table: AnyTable, report_type: ReportType) {
    spawn_workflow(
        gdata,
        None,
        move |session| session.olap_columns(report_type),
        move |columns| {
            columns_table.clear_table();
            for column in columns {
                columns_table.add_object(&BoxedAnyObject::new(column));
            }
        },
    );
}

fn run_report(
    gdata: Arc<GlobalData>,
    button: &Button,
    controls: &ReportControls,
    olap_fields: &DraggableOlapFields,
    columns_table: &AnyTable,
    report_table: AnyTable,
) {
    let id_to_name = field_names(columns_table);
    let fields = olap_fields.selected();
    let pivot = PivotSpec::new(&fields);

    let request = OlapRequest {
        report_type: controls.selected_report(),
        build_summary: false,
        group_by_row_fields: fields.rows,
        group_by_col_fields: fields.cols,
        aggregate_fields: fields.aggregates,
        filters: indexmap::IndexMap::from([(
            String::from(Filter::OPEN_DATE_FIELD),
            controls.date_filter(),
        )]),
    };

    spawn_workflow(
        gdata.clone(),
        Some(button),
        move |session| session.olap(&request),
        move |olap| {
            let total = translate(gdata.language(), TOTAL);

            let (data, olap_layout) = match &pivot.pivot {
                Some((col, value)) => (
                    olap.to_pivot_table(&pivot.rows, col, value, total),
                    OlapLayout::Pivot,
                ),
                None => (
                    olap.to_table_grouped(&pivot.rows, GroupOptions::grouped(total)),
                    OlapLayout::Grouped,
                ),
            };

            report_table.set_olap_table(data, &id_to_name, olap_layout);
            report_table.add_final();
        },
    );
}
