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
pub struct Employee {
    pub id: String,
    pub code: String,
    pub name: String,
    pub login: Option<String>,
    pub main_role_id: String,
    pub roles_ids: Option<Vec<String>>,
    pub main_role_code: String,
    pub role_codes: Option<Vec<String>>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub birthday: Option<String>,
    pub note: Option<String>,
    pub card_number: Option<String>,
    pub taxpayer_id_number: Option<String>,
    pub snils: Option<String>,
    pub preferred_department_code: Option<u32>,
    pub department_codes_state: Option<CodesState>,
    pub responsibility_department_codes_state: Option<CodesState>,
    pub deleted: bool,
    pub personal_data_consent: bool,
    pub supplier: bool,
    pub employee: bool,
    pub client: bool,
    pub represents_store: bool,
    pub public_external_data: Option<String>,
}

impl Employee {
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
pub struct EmployeeList {
    #[serde(rename = "employee", default)]
    pub employees: Vec<Employee>,
}

impl IikoSession {
    pub fn employees(
        &self,
        include_deleted: bool,
        revision_from: i32,
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
