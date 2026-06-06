use std::error::Error;

use serde::Deserialize;

use crate::api::{api_request::{ApiArgs, ApiRequest}, cashshifts_list::SessionStatus};

pub struct CashShiftsPaymentsList {
    address: String,
    token: String,
    id: u32,
}

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
    pub actualSum: u32,
    pub originalSum: u32,
    pub editedPayAccountId: String,
    pub originalPayAccountId: String,
    pub payAgentId: String,
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
    pub paymenTypeId: Option<String>,
    pub sum: u32,
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

#[derive(Deserialize)]
pub enum PaymentGroup {
    CARD,
    CREDIT,
    PAYOUT,
    PAYIN,
}

impl CashShiftsPaymentsList {
    pub fn new<S: Into<String>>(address: S, token: S, id: u32) -> Self {
        Self {
            address: address.into(),
            token: token.into(),
            id,
        }
    }

    pub fn run(&self) -> Result<CashShiftsPayments, Box<dyn Error>>
    {
        let args = ApiArgs::new([("key", &self.token)]);
        ApiRequest::new(self.address.clone(), format!("/resto/api/v2/cashshifts/payments/{}", self.id), args).run::<CashShiftsPayments>()
    }
}
