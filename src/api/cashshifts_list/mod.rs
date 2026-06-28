use crate::api::{api_request::*, error::ClientError};

pub struct CashShiftsList<'a> {
    address: &'a str,
    token: &'a str,
    date_from: &'a str,
    date_to: &'a str,
    status: &'a str,
}

pub type CashShifts = Vec<CashShift>;

#[allow(nonstandard_style)]
#[derive(serde::Deserialize, Debug)]
pub enum SessionStatus {
    OPEN,
    CLOSED,
    ACCEPTED,
    UNACCEPTED,
    HASWARNINGS,
}

#[allow(nonstandard_style)]
#[derive(serde::Deserialize, Debug)]
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
    pub payOrders: f64,
    pub sumWriteoffOrders: u32,
    pub salesCash: f64,
    pub salesCredit: f64,
    pub salesCard: f64,
    pub payIn: f32,
    pub payOut: f32,
    pub payIncome: f64,
    pub cashRemain: Option<i64>,
    pub cashDiff: f64,
    pub sessionStatus: SessionStatus,
    pub conceptionId: Option<String>,
    pub pointOfSaleId: String,
}

impl<'a> CashShiftsList<'a> {
    pub fn new(
        address: &'a str,
        token: &'a str,
        date_from: &'a str,
        date_to: &'a str,
        status: &'a str,
    ) -> Self {
        Self {
            address,
            token,
            date_from,
            date_to,
            status,
        }
    }

    pub fn run(&self) -> Result<CashShifts, ClientError> {
        let args = ApiArgs::new([
            ("key", self.token),
            ("openDateFrom", self.date_from),
            ("openDateTo", self.date_to),
            ("status", self.status),
        ]);
        let mut result: CashShifts =
            ApiRequest::new(self.address, "/resto/api/v2/cashshifts/list", args).run()?;
        result.sort_by_key(|shift| shift.sessionNumber);

        Ok(result)
    }
}
