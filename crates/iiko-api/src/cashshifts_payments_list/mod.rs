use serde::Deserialize;

use crate::{IikoSession, cashshifts_list::SessionStatus, error::ClientError};

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CashShiftsPayments {
    pub session_id: String,
    pub operation_day: String,
    pub cashless_records: Vec<CashShiftsPayment>,
    pub pay_in_records: Vec<CashShiftsPayment>,
    pub pay_outs_records: Vec<CashShiftsPayment>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CashShiftsPayment {
    pub info: PaymentInfo,
    pub actual_sum: f64,
    pub original_sum: f64,
    pub edited_pay_account_id: String,
    pub original_pay_account_id: String,
    pub pay_agent_id: Option<String>,
    pub payment_type_id: Option<String>,
    pub editable_comment: Option<String>,
    pub status: SessionStatus,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInfo {
    pub id: String,
    pub date: String,
    pub creation_date: String,
    pub group: PaymentGroup,
    pub account_id: String,
    pub payment_type_id: Option<String>,
    pub sum: f64,
    pub comment: Option<String>,
    pub auth: PaymentAuth,
    pub cause_event_id: String,
    pub cashier_id: String,
    pub department_id: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct PaymentAuth {
    pub user: String,
    pub card: String,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum PaymentGroup {
    Card,
    Credit,
    Payout,
    Payin,
}

impl PaymentGroup {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Card => "CARD",
            Self::Credit => "CREDIT",
            Self::Payout => "PAYOUT",
            Self::Payin => "PAYIN",
        }
    }
}

impl std::fmt::Display for PaymentGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl IikoSession {
    pub fn cashshifts_payments_list(
        &self,
        id: &str,
        hide_accepted: bool,
    ) -> Result<CashShiftsPayments, ClientError> {
        self.request_json(
            &format!("/resto/api/v2/cashshifts/payments/list/{id}"),
            &[("hideAccepted", &hide_accepted.to_string())],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{KEY, session};
    use httpmock::prelude::*;

    const CASH_SHIFTS_PAYMENTS_ANSWER: &str = include_str!("../../tests/cashshifts_payments.json");
    const ID: &str = "a1f3c2e0-5b7d-4c8a-9e21-0d4f6b8a1c33";

    #[test]
    fn cashshifts_payments_list_get() {
        let server = httpmock::MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/resto/api/v2/cashshifts/payments/list/{ID}"))
                .query_param("key", KEY)
                .query_param("hideAccepted", "false");
            then.status(200).body(CASH_SHIFTS_PAYMENTS_ANSWER);
        });

        let session = session(&server.base_url());

        let answer = session.cashshifts_payments_list(ID, false).unwrap();

        mock.assert();

        assert_eq!(
            answer,
            serde_json::from_str::<CashShiftsPayments>(CASH_SHIFTS_PAYMENTS_ANSWER).unwrap()
        );
    }
}
