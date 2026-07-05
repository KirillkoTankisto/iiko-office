use std::fmt::Display;

use serde::Deserialize;

use crate::{IikoSession, error::ClientError};

pub type CashShifts = Vec<CashShift>;

#[derive(Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SessionStatus {
    Any,
    Open,
    Closed,
    Accepted,
    Unaccepted,
    HasWarnings,
}

impl Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => f.write_str("ANY"),
            Self::Open => f.write_str("OPEN"),
            Self::Closed => f.write_str("CLOSED"),
            Self::Accepted => f.write_str("ACCEPTED"),
            Self::Unaccepted => f.write_str("UNACCEPTED"),
            Self::HasWarnings => f.write_str("HASWARNINGS"),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashShift {
    pub id: String,
    pub session_number: u32,
    pub fiscal_number: Option<u32>,
    pub cash_reg_number: u32,
    pub cash_reg_serial: String,
    pub open_date: String,
    pub close_date: Option<String>,
    pub accept_date: Option<String>,
    pub manager_id: String,
    pub session_start_cash: f64,
    pub pay_orders: f64,
    pub sum_writeoff_orders: f64,
    pub sales_cash: f64,
    pub sales_credit: f64,
    pub sales_card: f64,
    pub pay_in: f64,
    pub pay_out: f64,
    pub pay_income: f64,
    pub cash_remain: Option<i64>,
    pub cash_diff: f64,
    pub session_status: SessionStatus,
    pub conception_id: Option<String>,
    pub point_of_sale_id: String,
}

impl IikoSession {
    pub fn cashshifts_list(
        &self,
        from: &str,
        to: &str,
        session_status: SessionStatus,
    ) -> Result<CashShifts, ClientError> {
        self.request_json(
            "/resto/api/v2/cashshifts/list",
            &[
                ("openDateFrom", from),
                ("openDateTo", to),
                ("status", &session_status.to_string()),
            ],
        )
    }
}
