use crate::api::api_request::*;

pub struct CashShiftsList {
    address: String,
    token: String,
    date_from: String,
    date_to: String,
    status: String,
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

impl CashShiftsList {
    pub fn new(
        address: impl Into<String>,
        token: impl Into<String>,
        date_from: impl Into<String>,
        date_to: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
        Self {
            address: address.into(),
            token: token.into(),
            date_from: date_from.into(),
            date_to: date_to.into(),
            status: status.into(),
        }
    }

    pub fn run(&self) -> Result<CashShifts, String> {
        let args = ApiArgs::new([
            ("key", &self.token),
            ("openDateFrom", &self.date_from),
            ("openDateTo", &self.date_to),
            ("status", &self.status),
        ]);
        let mut result: CashShifts =
            ApiRequest::new(self.address.clone(), "/resto/api/v2/cashshifts/list", args).run()?;
        result.sort_by_key(|shift| shift.sessionNumber);

        Ok(result)
    }
}
