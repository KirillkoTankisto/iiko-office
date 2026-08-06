use std::fmt::Display;

use serde::Deserialize;

use crate::{IikoSession, error::ClientError};

pub type CashShifts = Vec<CashShift>;

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum SessionStatus {
    Any,
    Open,
    Closed,
    Accepted,
    Unaccepted,
    HasWarnings,
}

impl SessionStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Any => "ANY",
            Self::Open => "OPEN",
            Self::Closed => "CLOSED",
            Self::Accepted => "ACCEPTED",
            Self::Unaccepted => "UNACCEPTED",
            Self::HasWarnings => "HASWARNINGS",
        }
    }
}

impl Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Deserialize, PartialEq, Debug)]
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
    pub cash_remain: Option<f64>,
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
                ("status", session_status.as_str()),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{KEY, session};
    use httpmock::prelude::*;

    const CASH_SHIFTS_ANSWER: &str = include_str!("../../tests/cashshifts.json");
    const DATE_FROM: &str = "2026-01-01";
    const DATE_TO: &str = "2026-12-01";
    const STATUS: &str = "ANY";

    #[test]
    fn cashshifts_list_get() {
        let server = httpmock::MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/resto/api/v2/cashshifts/list")
                .query_param("key", KEY)
                .query_param("openDateFrom", DATE_FROM)
                .query_param("openDateTo", DATE_TO)
                .query_param("status", STATUS);
            then.status(200).body(CASH_SHIFTS_ANSWER);
        });

        let session = session(&server.base_url());

        let answer = session
            .cashshifts_list(DATE_FROM, DATE_TO, SessionStatus::Any)
            .unwrap();

        mock.assert();

        assert_eq!(
            serde_json::from_str::<CashShifts>(CASH_SHIFTS_ANSWER).unwrap(),
            answer
        );
    }
}
