use reqwest::{Url, blocking::Client};
use serde::de::DeserializeOwned;

use crate::api::error::ClientError;

const UAGENT: &str = concat!("iiko-office-libre/", env!("CARGO_PKG_VERSION"));

pub struct ApiArgs<'a, const N: usize> {
    value: [(&'a str, &'a str); N],
}

impl<'a, const N: usize> ApiArgs<'a, N> {
    pub fn new(value: [(&'a str, &'a str); N]) -> Self {
        Self { value }
    }
}

pub struct ApiRequest<'a, const N: usize> {
    address: &'a str,
    path: &'a str,
    args: ApiArgs<'a, N>,
}

impl<'a, const N: usize> ApiRequest<'a, N> {
    pub fn new(address: &'a str, path: &'a str, args: ApiArgs<'a, N>) -> Self {
        Self {
            address,
            path,
            args,
        }
    }

    pub fn run<T: DeserializeOwned>(&self) -> Result<T, ClientError> {
        let client = build_client()?;

        let url = parse_url(self.address, self.path, &self.args)?;

        let result: T = client.get(url).send()?.error_for_status()?.json()?;

        Ok(result)
    }

    pub fn run_string(&self) -> Result<String, ClientError> {
        let client = build_client()?;

        let url = parse_url(self.address, self.path, &self.args)?;

        let result: String = client.get(url).send()?.error_for_status()?.text()?;

        Ok(result)
    }

    pub fn run_xml<T: DeserializeOwned>(&self) -> Result<T, ClientError> {
        let result = Self::run_string(self)?;
        Ok(quick_xml::de::from_str(&result)?)
    }

    pub fn run_post<T: DeserializeOwned>(&self, data: String) -> Result<T, ClientError> {
        let client = build_client()?;

        let url = parse_url(self.address, self.path, &self.args)?;

        let result: T = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(data)
            .send()?
            .error_for_status()?
            .json()?;

        Ok(result)
    }
}

fn build_client() -> Result<Client, ClientError> {
    Ok(Client::builder().user_agent(UAGENT).build()?)
}

fn parse_url<const N: usize>(
    address: &str,
    path: &str,
    args: &ApiArgs<N>,
) -> Result<Url, url::ParseError> {
    let mut url: Url = Url::parse(address)?;
    url.set_path(path);
    url.query_pairs_mut().extend_pairs(&args.value);
    Ok(url)
}
