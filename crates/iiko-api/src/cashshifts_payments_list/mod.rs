use serde::Deserialize;
use strum_macros::Display;

use crate::{IikoSession, cashshifts_list::SessionStatus, error::ClientError};

#[derive(Deserialize, PartialEq, Debug)]
#[allow(nonstandard_style)]
pub struct CashShiftsPayments {
    pub sessionId: String,
    pub operationDay: String,
    pub cashlessRecords: Vec<CashShiftsPayment>,
    pub payInRecords: Vec<CashShiftsPayment>,
    pub payOutsRecords: Vec<CashShiftsPayment>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[allow(nonstandard_style)]
pub struct CashShiftsPayment {
    pub info: PaymentInfo,
    pub actualSum: f64,
    pub originalSum: f64,
    pub editedPayAccountId: String,
    pub originalPayAccountId: String,
    pub payAgentId: Option<String>,
    pub paymentTypeId: Option<String>,
    pub editableComment: Option<String>,
    pub status: SessionStatus,
}

#[derive(Deserialize, PartialEq, Debug)]
#[allow(nonstandard_style)]
pub struct PaymentInfo {
    pub id: String,
    pub date: String,
    pub creationDate: String,
    pub group: PaymentGroup,
    pub accountId: String,
    pub paymentTypeId: Option<String>,
    pub sum: f64,
    pub comment: Option<String>,
    pub auth: PaymentAuth,
    pub causeEventId: String,
    pub cashierId: String,
    pub departmentId: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct PaymentAuth {
    pub user: String,
    pub card: String,
}

#[derive(Deserialize, Display, PartialEq, Debug)]
pub enum PaymentGroup {
    CARD,
    CREDIT,
    PAYOUT,
    PAYIN,
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
    use crate::IikoConnection;
    use httpmock::prelude::*;
    use std::sync::Mutex;

    const CASH_SHIFTS_PAYMENTS_ANSWER: &str = include_str!("../../tests/cashshifts_payments.json");
    const ID: &str = "a1f3c2e0-5b7d-4c8a-9e21-0d4f6b8a1c33";
    const KEY: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
    const PASSWORD: &str = "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8";
    const USER: &str = "admin";

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

        let session = IikoSession {
            connection: IikoConnection::new(&server.base_url()).unwrap(),
            user: USER.to_string(),
            hashed_password: PASSWORD.to_string(),
            token: Mutex::new(KEY.to_string()),
        };

        let answer = session.cashshifts_payments_list(ID, false).unwrap();

        mock.assert();

        assert_eq!(
            answer,
            serde_json::from_str::<CashShiftsPayments>(CASH_SHIFTS_PAYMENTS_ANSWER).unwrap()
        );
    }
}
