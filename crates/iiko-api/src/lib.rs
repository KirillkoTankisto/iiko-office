pub mod auth;
pub mod cashshifts_list;
pub mod cashshifts_payments_list;
pub mod consts;
pub mod error;
pub mod logout;
pub mod olap;
pub mod olap_columns;
pub mod utils;
pub mod version;

use std::{sync::Mutex, time::Duration};

use reqwest::StatusCode;
use serde::de::DeserializeOwned;

use crate::error::ClientError;

const UAGENT: &str = concat!("iiko-office-libre/", env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
pub struct IikoConnection {
    client: reqwest::blocking::Client,
    base: url::Url,
}

fn check_status(
    resp: reqwest::blocking::Response,
) -> Result<reqwest::blocking::Response, ClientError> {
    match resp.status() {
        s if s == StatusCode::UNAUTHORIZED || s == StatusCode::FORBIDDEN => {
            Err(ClientError::Unauthorized)
        }
        _ => Ok(resp.error_for_status().map_err(|e| e.without_url())?),
    }
}

impl IikoConnection {
    pub fn new(address: &str) -> Result<Self, ClientError> {
        let client = reqwest::blocking::Client::builder()
            .user_agent(UAGENT)
            .timeout(Duration::from_secs(10))
            .build()?;
        let base = url::Url::parse(address)?;

        Ok(Self { client, base })
    }

    fn get(
        &self,
        path: &str,
        args: &[(&str, &str)],
    ) -> Result<reqwest::blocking::Response, ClientError> {
        let resp = self.client.get(self.url(path, args)).send()?;
        check_status(resp)
    }

    fn request_string(&self, path: &str, args: &[(&str, &str)]) -> Result<String, ClientError> {
        Ok(self.get(path, args)?.text()?)
    }

    fn request_json<T: DeserializeOwned>(
        &self,
        path: &str,
        args: &[(&str, &str)],
    ) -> Result<T, ClientError> {
        Ok(self.get(path, args)?.json()?)
    }

    fn request_xml<T: DeserializeOwned>(
        &self,
        path: &str,
        args: &[(&str, &str)],
    ) -> Result<T, ClientError> {
        Ok(quick_xml::de::from_str(&self.get(path, args)?.text()?)?)
    }

    // Supports only json POST
    fn request_post<T: DeserializeOwned>(
        &self,
        path: &str,
        args: &[(&str, &str)],
        data: String,
    ) -> Result<T, ClientError> {
        let resp = self
            .client
            .post(self.url(path, args))
            .header("Content-Type", "application/json")
            .body(data)
            .send()?;
        Ok(check_status(resp)?.json()?)
    }

    #[inline]
    fn url(&self, path: &str, args: &[(&str, &str)]) -> url::Url {
        let mut url = self.base.clone();
        url.set_path(path);
        url.query_pairs_mut().extend_pairs(args).finish();
        url
    }
}

#[derive(Debug)]
pub struct IikoSession {
    connection: IikoConnection,
    user: String,
    hashed_password: String,
    token: Mutex<String>,
}

impl IikoSession {
    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn reauth(&self) -> Result<(), ClientError> {
        let mut token = self.token.lock().unwrap_or_else(|e| e.into_inner());
        *token = self.connection.request_string(
            "/resto/api/auth",
            &[("login", &self.user), ("pass", &self.hashed_password)],
        )?;
        Ok(())
    }

    fn token(&self) -> String {
        self.token.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    fn with_key<T>(
        &self,
        args: &[(&str, &str)],
        call: impl Fn(&[(&str, &str)]) -> Result<T, ClientError>,
    ) -> Result<T, ClientError> {
        let run = |token: &str| {
            let mut full_args = vec![("key", token)];
            full_args.extend_from_slice(args);
            call(&full_args)
        };

        match run(&self.token()) {
            Err(ClientError::Unauthorized) => {
                self.reauth()?;
                run(&self.token())
            }
            other => other,
        }
    }

    fn request_string(&self, path: &str, args: &[(&str, &str)]) -> Result<String, ClientError> {
        self.with_key(args, |args| self.connection.request_string(path, args))
    }

    fn request_json<T: DeserializeOwned>(
        &self,
        path: &str,
        args: &[(&str, &str)],
    ) -> Result<T, ClientError> {
        self.with_key(args, |args| self.connection.request_json(path, args))
    }

    fn request_post<T: DeserializeOwned>(
        &self,
        path: &str,
        args: &[(&str, &str)],
        data: String,
    ) -> Result<T, ClientError> {
        self.with_key(args, |args| {
            self.connection.request_post(path, args, data.clone())
        })
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    use super::*;

    pub const KEY: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
    pub const PASSWORD: &str = "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8";
    pub const USER: &str = "admin";

    /// A session pointing at a mock server, already holding [`KEY`].
    pub fn session(base_url: &str) -> IikoSession {
        IikoSession {
            connection: IikoConnection::new(base_url).unwrap(),
            user: USER.to_string(),
            hashed_password: PASSWORD.to_string(),
            token: Mutex::new(KEY.to_string()),
        }
    }
}
