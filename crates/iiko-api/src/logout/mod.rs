use crate::{IikoSession, error::ClientError};

impl IikoSession {
    pub fn logout(&self) -> Result<(), ClientError> {
        self.request_string("/resto/api/logout", &[]).map(|_| ())
    }
}
