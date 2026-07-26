use std::sync::Mutex;

use crate::{IikoConnection, IikoSession, error::ClientError};

impl IikoConnection {
    pub fn auth(self, login: &str, password: &str) -> Result<IikoSession, ClientError> {
        let token =
            self.request_string("/resto/api/auth", &[("login", login), ("pass", password)])?;

        Ok(IikoSession {
            connection: self,
            user: login.to_string(),
            hashed_password: password.to_string(),
            token: Mutex::new(token),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use std::assert_matches;

    #[test]
    fn auth_correct_password() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/resto/api/auth")
                .query_param("login", "admin")
                .query_param("pass", "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8");
            then.status(200)
                .body("da39a3ee5e6b4b0d3255bfef95601890afd80709");
        });

        let connection = IikoConnection::new(&server.base_url()).unwrap();
        let session = connection
            .auth("admin", "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8")
            .unwrap();
        mock.assert();

        assert_eq!(session.user, "admin");
        assert_eq!(session.token(), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn auth_incorrect_password() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/resto/api/auth");
            then.status(401).body("invalid password");
        });

        let connection = IikoConnection::new(&server.base_url()).unwrap();

        assert_matches!(
            connection.auth("wrong", "password").unwrap_err(),
            ClientError::Unauthorized
        );
    }
}
