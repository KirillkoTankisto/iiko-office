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

use std::sync::Mutex;

use reqwest::StatusCode;
use serde::de::DeserializeOwned;

use crate::error::ClientError;

const UAGENT: &str = concat!("iiko-office-libre/", env!("CARGO_PKG_VERSION"));

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
            .build()?;
        let base = url::Url::parse(address)?;

        Ok(Self { client, base })
    }

    fn request_string(&self, path: &str, args: &[(&str, &str)]) -> Result<String, ClientError> {
        let resp = self
            .client
            .get(Self::parse_url(&self.base, path, args))
            .send()?;

        Ok(check_status(resp)?.text()?)
    }

    fn request_json<T: DeserializeOwned>(
        &self,
        path: &str,
        args: &[(&str, &str)],
    ) -> Result<T, ClientError> {
        let resp = self
            .client
            .get(Self::parse_url(&self.base, path, args))
            .send()?;

        Ok(check_status(resp)?.json()?)
    }

    fn request_xml<T: DeserializeOwned>(
        &self,
        path: &str,
        args: &[(&str, &str)],
    ) -> Result<T, ClientError> {
        let resp = self
            .client
            .get(Self::parse_url(&self.base, path, args))
            .send()?;
        Ok(quick_xml::de::from_str(&check_status(resp)?.text()?)?)
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
            .post(Self::parse_url(&self.base, path, args))
            .header("Content-Type", "application/json")
            .body(data)
            .send()?;
        Ok(check_status(resp)?.json()?)
    }

    #[inline]
    fn parse_url(base: &url::Url, path: &str, args: &[(&str, &str)]) -> url::Url {
        let mut base = base.clone();
        base.set_path(path);
        base.query_pairs_mut().extend_pairs(args).finish();
        base
    }
}

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

    fn with_reauth<T>(
        &self,
        attempt: impl Fn(&str) -> Result<T, ClientError>,
    ) -> Result<T, ClientError> {
        match attempt(&self.token()) {
            Err(ClientError::Unauthorized) => {
                self.reauth()?;
                attempt(&self.token())
            }
            other => other,
        }
    }

    fn request_string(&self, path: &str, args: &[(&str, &str)]) -> Result<String, ClientError> {
        self.with_reauth(|token| {
            let mut full_args: Vec<(&str, &str)> = vec![("key", token)];
            full_args.extend_from_slice(args);
            self.connection.request_string(path, &full_args)
        })
    }

    fn request_json<T: DeserializeOwned>(
        &self,
        path: &str,
        args: &[(&str, &str)],
    ) -> Result<T, ClientError> {
        self.with_reauth(|token| {
            let mut full_args: Vec<(&str, &str)> = vec![("key", token)];
            full_args.extend_from_slice(args);
            self.connection.request_json(path, &full_args)
        })
    }

    #[allow(unused)]
    fn request_xml<T: DeserializeOwned>(
        &self,
        path: &str,
        args: &[(&str, &str)],
    ) -> Result<T, ClientError> {
        self.with_reauth(|token| {
            let mut full_args: Vec<(&str, &str)> = vec![("key", token)];
            full_args.extend_from_slice(args);
            self.connection.request_xml(path, &full_args)
        })
    }

    fn request_post<T: DeserializeOwned>(
        &self,
        path: &str,
        args: &[(&str, &str)],
        data: String,
    ) -> Result<T, ClientError> {
        self.with_reauth(|token| {
            let mut full_args: Vec<(&str, &str)> = vec![("key", token)];
            full_args.extend_from_slice(args);
            self.connection.request_post(path, &full_args, data.clone())
        })
    }
}
