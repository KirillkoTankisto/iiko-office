use serde::Deserialize;
use strum_macros::Display;

use crate::{IikoSession, cashshifts_list::SessionStatus, error::ClientError};

#[derive(Deserialize)]
#[allow(nonstandard_style)]
pub struct CashShiftsPayments {
    pub sessionId: String,
    pub operationDay: String,
    pub cashlessRecords: Vec<CashShiftsPayment>,
    pub payInRecords: Vec<CashShiftsPayment>,
    pub payOutsRecords: Vec<CashShiftsPayment>,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct PaymentAuth {
    pub user: String,
    pub card: String,
}

#[derive(Deserialize, Display)]
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
