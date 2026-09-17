use std::sync::Arc;

use gtk4::{Align, prelude::*};
use iiko_api::{consts::AsStr, employees::Employee};

use crate::gui::{
    common::{
        anybox::AnyBox,
        global_data::GlobalData,
        table::{AnyTable, ColumnSpec, GetTable},
        utils::spawn_workflow,
    },
    main::menu::{tabs::AnyTab, view::MainView},
    translation::{CurrentLanguage, Line, translate},
};

pub struct EmployeesTab;

impl AnyTab for EmployeesTab {
    fn title(&self, gdata: &GlobalData) -> &str {
        translate(gdata.language(), Line::EMPLOYEES)
    }

    fn build(&self, gdata: Arc<GlobalData>, _view: &MainView) -> gtk4::Widget {
        let employees_table: AnyTable<Employee> = Self::get_table(gdata.language());

        let root = AnyBox::horizontal()
            .margin(8)
            .add_widgets([employees_table.present().upcast_ref()])
            .consume()
            .upcast();

        spawn_workflow(
            gdata,
            None,
            |session| session.employees(false, -1),
            move |employees| {
                for employee in employees {
                    employees_table.add_object(employee);
                }
            },
        );

        root
    }
}

const COLUMNS: &[ColumnSpec<Employee>] = &[
    ColumnSpec::new(Line::EMPLOYEE_NAME, Align::Start, |e| e.name.clone()),
    ColumnSpec::new(Line::EMPLOYEE_MAIN_ROLE, Align::Start, |e| {
        e.main_role_code.clone()
    }),
    ColumnSpec::new(Line::EMPLOYEE_FULL_NAME, Align::Start, |e| {
        e.full_name().unwrap_or_default()
    }),
    ColumnSpec::new(Line::EMPLOYEE_NOTE, Align::Start, |e| {
        e.note.to_owned().unwrap_or_default()
    }),
    ColumnSpec::new(Line::EMPLOYEE_LOGIN, Align::Start, |e| {
        e.login.to_owned().unwrap_or_default()
    }),
    ColumnSpec::new(Line::EMPLOYEE_SUPPLIER, Align::Start, |e| {
        e.supplier.as_str().to_owned()
    }),
    ColumnSpec::new(Line::EMPLOYEE_EMPLOYEE, Align::Start, |e| {
        e.employee.as_str().to_owned()
    }),
    ColumnSpec::new(Line::EMPLOYEE_CLIENT, Align::Start, |e| {
        e.client.as_str().to_owned()
    }),
    ColumnSpec::new(Line::EMPLOYEE_REPRESENTS_STORE, Align::Start, |e| {
        e.represents_store.as_str().to_owned()
    }),
];

impl GetTable<Employee> for EmployeesTab {
    fn get_table(language: CurrentLanguage) -> AnyTable<Employee> {
        let employees_table: AnyTable<Employee> = AnyTable::new(true);

        employees_table.add_columns(language, COLUMNS);
        employees_table.add_final();

        employees_table
    }
}
