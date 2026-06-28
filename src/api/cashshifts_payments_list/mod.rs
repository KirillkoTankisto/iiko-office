use serde::Deserialize;
use strum_macros::Display;

use crate::api::{
    api_request::{ApiArgs, ApiRequest},
    cashshifts_list::SessionStatus,
    error::ClientError,
};

pub struct CashShiftsPaymentsList<'a> {
    address: &'a str,
    token: &'a str,
    id: &'a str,
}

#[derive(Deserialize, Debug)]
#[allow(nonstandard_style)]
pub struct CashShiftsPayments {
    pub sessionId: String,
    pub operationDay: String,
    pub cashlessRecords: Vec<CashShiftsPayment>,
    pub payInRecords: Vec<CashShiftsPayment>,
    pub payOutsRecords: Vec<CashShiftsPayment>,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct PaymentAuth {
    pub user: String,
    pub card: String,
}

#[derive(Deserialize, Debug, Display)]
pub enum PaymentGroup {
    CARD,
    CREDIT,
    PAYOUT,
    PAYIN,
}

impl<'a> CashShiftsPaymentsList<'a> {
    pub fn new(address: &'a str, token: &'a str, id: &'a str) -> Self {
        Self { address, token, id }
    }

    pub fn run(&self) -> Result<CashShiftsPayments, ClientError> {
        let args = ApiArgs::new([("key", self.token), ("hideAccepted", "false")]);
        ApiRequest::new(
            self.address,
            &format!("/resto/api/v2/cashshifts/payments/list/{}", self.id),
            args,
        )
        .run()
    }
}
