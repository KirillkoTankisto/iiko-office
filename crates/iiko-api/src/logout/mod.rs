use crate::{IikoSession, error::ClientError};

impl IikoSession {
    pub fn logout(&self) -> Result<(), ClientError> {
        self.request_string("/resto/api/logout", &[]).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::IikoConnection;

    use super::*;
    use httpmock::prelude::*;

    const KEY: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
    const PASSWORD: &str = "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8";
    const USER: &str = "admin";

    #[test]
    fn logout_test() {
        let server = httpmock::MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/resto/api/logout")
                .query_param("key", KEY);
            then.status(200).body("logout success");
        });

        let session = IikoSession {
            connection: IikoConnection::new(&server.base_url()).unwrap(),
            user: USER.to_string(),
            hashed_password: PASSWORD.to_string(),
            token: Mutex::new(KEY.to_string()),
        };

        session.logout().unwrap();

        mock.assert();
    }
}
