use crate::api::api_request::*;
use std::error::Error;

pub struct CashShiftsList {
    address: String,
    token: String,
    date_from: String,
    date_to: String,
    status: String,
}

pub type CashShifts = Vec<CashShift>;

#[allow(nonstandard_style)]
#[derive(serde::Deserialize)]
#[derive(Debug)]
pub enum SessionStatus {
    OPEN,
    CLOSED,
    ACCEPTED,
    UNACCEPTED,
    HASWARNINGS,
}

#[allow(nonstandard_style)]
#[derive(serde::Deserialize)]
#[derive(Debug)]
pub struct CashShift {
    pub id: String,
    pub sessionNumber: u32,
    pub fiscalNumber: Option<u32>,
    pub cashRegNumber: u32,
    pub cashRegSerial: String,
    pub openDate: String,
    pub closeDate: Option<String>,
    pub acceptDate: Option<String>,
    pub managerId: String,
    pub sessionStartCash: u32,
    pub payOrders: u32,
    pub sumWriteoffOrders: u32,
    pub salesCash: u32,
    pub salesCredit: u32,
    pub salesCard: u32,
    pub payIn: u32,
    pub payOut: u32,
    pub payIncome: i64,
    pub cashRemain: Option<i64>,
    pub cashDiff: i64,
    pub sessionStatus: SessionStatus,
    pub conceptionId: Option<String>,
    pub pointOfSaleId: String,
}

impl CashShiftsList {
    pub fn new<S: Into<String>>(address: S, token: S, date_from: S, date_to: S, status: S) -> Self {
        Self {
            address: address.into(),
            token: token.into(),
            date_from: date_from.into(),
            date_to: date_to.into(),
            status: status.into(),
        }
    }

    pub fn run(&self) -> Result<CashShifts, Box<dyn Error>> {
        let args = ApiArgs::new([
            ("key", &self.token),
            ("openDateFrom", &self.date_from),
            ("openDateTo", &self.date_to),
            ("status", &self.status),
        ]);
        let mut result: CashShifts = ApiRequest::new(
            self.address.clone(),
            "/resto/api/v2/cashshifts/list".into(),
            args,
        )
        .run()?;
        result.sort_by_key(|shift| shift.sessionNumber);

        Ok(result)
    }
}
