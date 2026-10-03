//! Библиотека для работы с iikoServer API
//!
//! Предоставляет объект iikoConnection и iikoSession для взаимодействия с iikoServer API

pub mod auth;
pub mod cashshifts_list;
pub mod cashshifts_payments_list;
pub mod consts;
pub mod employees;
pub mod error;
pub mod logout;
pub mod olap;
pub mod olap_columns;
pub mod utils;
pub mod version;

mod macros;

use std::{sync::Mutex, time::Duration};

use serde::de::DeserializeOwned;
use ureq::{
    Agent, Body,
    http::{Response, Uri},
    tls::{RootCerts, TlsConfig},
};

use crate::error::ClientError;

const UAGENT: &str = concat!("iiko-office-libre/", env!("CARGO_PKG_VERSION"));
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const MAX_BODY: u64 = 512 * 1024 * 1024;

#[derive(Debug)]
/// Клиент для взаимодействия с API без аутентификации
pub struct IikoConnection {
    agent: ureq::Agent,
    base: String,
}

fn check_status(
    resp: ureq::http::Response<ureq::Body>,
) -> Result<ureq::http::Response<ureq::Body>, ClientError> {
    match resp.status().as_u16() {
        200..=299 => Ok(resp),
        401 => Err(ClientError::Unauthorized),
        403 => Err(ClientError::Forbidden),
        code => Err(ClientError::Status(code)),
    }
}

fn read_string(mut resp: Response<Body>) -> Result<String, ClientError> {
    Ok(resp
        .body_mut()
        .with_config()
        .limit(MAX_BODY)
        .read_to_string()?)
}

fn read_json<T: DeserializeOwned>(mut resp: Response<Body>) -> Result<T, ClientError> {
    let bytes = resp
        .body_mut()
        .with_config()
        .limit(MAX_BODY)
        .read_to_vec()?;
    Ok(serde_json::from_slice(&bytes)?)
}

impl IikoConnection {
    /// Создаёт новый клиент.
    /// Выдаст ошибку при некорректном адресе
    pub fn new(address: &str) -> Result<Self, ClientError> {
        let uri: Uri = address.trim().parse().map_err(|_| ClientError::Address)?;

        let base = match (uri.scheme_str(), uri.authority()) {
            (Some(scheme @ ("http" | "https")), Some(authority)) => {
                format!("{scheme}://{authority}")
            }
            _ => return Err(ClientError::Address),
        };

        let agent = Agent::config_builder()
            .user_agent(UAGENT)
            .http_status_as_error(false)
            .timeout_connect(Some(CONNECT_TIMEOUT))
            .timeout_global(Some(REQUEST_TIMEOUT))
            .tls_config(
                TlsConfig::builder()
                    .root_certs(RootCerts::PlatformVerifier)
                    .build(),
            )
            .build()
            .into();

        Ok(Self { agent, base })
    }

    #[inline]
    fn url(&self, path: &str) -> String {
        debug_assert!(path.starts_with('/'));
        format!("{}{}", self.base, path)
    }

    fn get(&self, path: &str, args: &[(&str, &str)]) -> Result<Response<Body>, ClientError> {
        let resp = self
            .agent
            .get(self.url(path))
            .query_pairs(args.iter().copied()) // percent-encoded by ureq
            .call()?;
        check_status(resp)
    }

    fn request_string(&self, path: &str, args: &[(&str, &str)]) -> Result<String, ClientError> {
        read_string(self.get(path, args)?)
    }

    fn request_json<T: DeserializeOwned>(
        &self,
        path: &str,
        args: &[(&str, &str)],
    ) -> Result<T, ClientError> {
        read_json(self.get(path, args)?)
    }

    fn request_xml<T: DeserializeOwned>(
        &self,
        path: &str,
        args: &[(&str, &str)],
    ) -> Result<T, ClientError> {
        Ok(quick_xml::de::from_str(&self.request_string(path, args)?)?)
    }

    // Supports only json POST
    fn request_post<T: DeserializeOwned>(
        &self,
        path: &str,
        args: &[(&str, &str)],
        data: String,
    ) -> Result<T, ClientError> {
        let resp = self
            .agent
            .post(self.url(path))
            .query_pairs(args.iter().copied())
            .header("Content-Type", "application/json")
            .send(data)?;
        read_json(check_status(resp)?)
    }
}

#[derive(Debug)]
/// Сессия, которая хранит клиент и данные для авторизации
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
}

/// Gives IikoSession the same calls as IikoConnection,
/// with api key added to args
macro_rules! forward_with_key {
    ($( $name:ident $(<$generic:ident>)? ($($arg:ident: $argty:ty),*) -> $ret:ty ),+ $(,)?) => {
        impl IikoSession {
            $(
                fn $name $(<$generic: DeserializeOwned>)? (
                    &self,
                    path: &str,
                    args: &[(&str, &str)],
                    $($arg: $argty),*
                ) -> Result<$ret, ClientError> {
                    self.with_key(args, |args| {
                        self.connection.$name(path, args $(, $arg.clone())*)
                    })
                }
            )+
        }
    };
}

forward_with_key! {
    request_string() -> String,
    request_json<T>() -> T,
    request_xml<T>() -> T,
    request_post<T>(data: String) -> T,
}

#[cfg(test)]
pub(crate) mod test_utils {
    use super::*;

    pub const KEY: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
    pub const PASSWORD: &str = "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8";
    pub const USER: &str = "admin";

    /// An IikoSession, with api key
    pub fn session(base_url: &str) -> IikoSession {
        IikoSession {
            connection: IikoConnection::new(base_url).unwrap(),
            user: USER.to_string(),
            hashed_password: PASSWORD.to_string(),
            token: Mutex::new(KEY.to_string()),
        }
    }
}
