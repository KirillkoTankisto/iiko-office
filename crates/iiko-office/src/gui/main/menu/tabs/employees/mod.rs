use gtk4::{glib::object::Cast, prelude::BoxExt};
use iiko_api::employees::Employee;

use crate::gui::{
    common::{
        table::{AnyTable, AsTable, ColumnSpec},
        utils::spawn_workflow,
    },
    main::menu::tabs::{AnyTab, build_box},
    translation::{Line, translate},
};

pub struct EmployeesTab;

impl AnyTab for EmployeesTab {
    fn title(&self, gdata: &crate::gui::common::global_data::GlobalData) -> &str {
        translate(gdata.language(), Line::EMPLOYEES)
    }

    fn build(
        &self,
        gdata: std::sync::Arc<crate::gui::common::global_data::GlobalData>,
        _view: &crate::gui::main::menu::view::MainView,
    ) -> gtk4::Widget {
        let root = build_box(gtk4::Orientation::Horizontal);

        let employees_table: AnyTable<Employee> = Self::as_table(gdata.language());

        root.append(employees_table.present());

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

        root.upcast()
    }
}

const COLUMNS: &[ColumnSpec<Employee>] = &[
    ColumnSpec::new(Line::MENUBAR_FILE, gtk4::Align::Start, |e| e.name.clone()),
    ColumnSpec::new(Line::MENUBAR_FILE, gtk4::Align::Start, |e| {
        e.main_role_code.clone()
    }),
    ColumnSpec::new(Line::MENUBAR_FILE, gtk4::Align::Start, |e| {
        e.full_name().unwrap_or_default()
    }),
    ColumnSpec::new(Line::MENUBAR_FILE, gtk4::Align::Start, |e| {
        e.note.to_owned().unwrap_or_default()
    }),
];

impl AsTable<Employee> for EmployeesTab {
    fn as_table(language: crate::gui::translation::CurrentLanguage) -> AnyTable<Employee> {
        let employees_table: AnyTable<Employee> = AnyTable::new(true);

        employees_table.add_columns(language, COLUMNS);
        employees_table.add_final();

        employees_table
    }
}
