//! Список сотрудников

use serde::Deserialize;

use crate::{IikoSession, consts::AsStr, error::ClientError, macros::str_enum};

str_enum! {
    pub enum CodesState {
        Empty => "EMPTY",
        Null => "NULL",
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
/// Данные о сотруднике
pub struct Employee {
    pub id: String,
    pub code: String,
    pub name: String,
    pub login: Option<String>,
    pub password: Option<String>,
    pub main_role_id: Option<String>,
    pub roles_ids: Option<Vec<String>>,
    pub main_role_code: Option<String>,
    pub role_codes: Option<Vec<String>>,
    pub phone: Option<String>,
    pub cell_phone: Option<String>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub birthday: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub hire_date: Option<String>,
    pub hire_document_number: Option<String>,
    pub fire_date: Option<String>,
    pub note: Option<String>,
    pub card_number: Option<String>,
    pub pin_code: Option<String>,
    pub taxpayer_id_number: Option<String>,
    pub snils: Option<String>,
    pub gln: Option<String>,
    pub activation_date: Option<String>,
    pub deactivation_date: Option<String>,
    pub preferred_department_code: Option<String>,
    pub department_codes: Option<Vec<String>>,
    pub responsibility_department_codes: Option<Vec<String>>,
    pub deleted: Option<bool>,
    pub supplier: Option<bool>,
    pub employee: Option<bool>,
    pub client: Option<bool>,
}

impl Employee {
    /// Получить полное имя сотрудника
    pub fn full_name(&self) -> Option<String> {
        let parts: Vec<&str> = [&self.first_name, &self.middle_name, &self.last_name]
            .into_iter()
            .flatten()
            .map(String::as_str)
            .collect();

        (!parts.is_empty()).then(|| parts.join(" "))
    }
}

#[derive(Deserialize)]
/// приватный, нужен для правильного парсинга ответа
struct EmployeeList {
    #[serde(rename = "employee", default)]
    pub employees: Vec<Employee>,
}

impl IikoSession {
    /// Получить список сотрудников
    pub fn employees(
        &self,
        include_deleted: bool,
        revision_from: i32, // -1 for all
    ) -> Result<Vec<Employee>, ClientError> {
        let result: EmployeeList = self.request_xml(
            "/resto/api/employees",
            &[
                ("includeDeleted", include_deleted.as_str()),
                ("revisionFrom", &revision_from.to_string()),
            ],
        )?;

        Ok(result.employees)
    }
}
