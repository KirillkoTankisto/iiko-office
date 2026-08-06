use serde::Deserialize;

use crate::{IikoSession, error::ClientError};

#[derive(Deserialize)]
pub enum CodesState {
    EMPTY,
    NULL,
}

#[derive(Deserialize)]
pub struct Employee {
    pub id: String,
    pub code: u32,
    pub name: String,
    pub login: Option<String>,
    pub main_role_id: String,
    pub roles_ids: String,
    pub main_role_code: String,
    pub role_codes: Vec<String>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub birthday: Option<String>,
    pub note: String,
    pub card_number: Option<String>,
    pub taxpayer_id_number: Option<String>,
    pub snils: Option<String>,
    pub preferred_department_code: u32,
    pub department_codes_state: CodesState,
    pub responsibility_department_codes_state: CodesState,
    pub deleted: bool,
    pub personal_data_consent: bool,
    pub supplier: bool,
    pub employee: bool,
    pub client: bool,
    pub represents_store: bool,
    pub public_external_data: Option<String>
}

pub type EmployeeList = Vec<Employee>;

impl IikoSession {
    pub fn employees(&self, include_deleted: bool, revision_from: i32) -> Result<EmployeeList, ClientError> {
        self.request_xml("/resto/api/v2/employees", &[("includeDeleted", &include_deleted.to_string()), ("revisionFrom", &revision_from.to_string())])
    }
}
