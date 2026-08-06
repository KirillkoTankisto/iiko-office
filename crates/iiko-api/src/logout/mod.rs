use crate::{IikoSession, error::ClientError};

impl IikoSession {
    pub fn logout(&self) -> Result<(), ClientError> {
        self.request_string("/resto/api/logout", &[]).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{KEY, session};
    use httpmock::prelude::*;

    #[test]
    fn logout_test() {
        let server = httpmock::MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/resto/api/logout")
                .query_param("key", KEY);
            then.status(200).body("logout success");
        });

        let session = session(&server.base_url());

        session.logout().unwrap();

        mock.assert();
    }
}
