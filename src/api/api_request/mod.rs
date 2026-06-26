use reqwest::{Url, blocking::Client};
use serde::de::DeserializeOwned;

const UAGENT: &str = concat!("iiko-office-libre/", env!("CARGO_PKG_VERSION"));

pub struct ApiArgs<const N: usize> {
    value: [(String, String); N],
}

impl<const N: usize> ApiArgs<N> {
    pub fn new(input: [(impl Into<String>, impl Into<String>); N]) -> Self {
        Self {
            value: input.map(|(key, value)| (key.into(), value.into())),
        }
    }
}

pub struct ApiRequest<const N: usize> {
    address: String,
    path: String,
    args: ApiArgs<N>,
}

impl<const N: usize> ApiRequest<N> {
    pub fn new(address: impl Into<String>, path: impl Into<String>, args: ApiArgs<N>) -> Self {
        Self {
            address: address.into(),
            path: path.into(),
            args,
        }
    }

    pub fn run<T: DeserializeOwned>(&self) -> Result<T, String> {
        let client = build_client()?;

        let mut url: Url = Url::parse(&self.address).map_err(|e| e.to_string())?;
        url.set_path(&self.path);
        url.query_pairs_mut().extend_pairs(&self.args.value);

        let result: T = client
            .get(url)
            .send()
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .json()
            .map_err(|e| e.to_string())?;

        Ok(result)
    }

    pub fn run_string(&self) -> Result<String, String> {
        let client = build_client()?;

        let mut url: Url = Url::parse(&self.address).map_err(|e| e.to_string())?;
        url.set_path(&self.path);
        url.query_pairs_mut().extend_pairs(&self.args.value);

        let result: String = client
            .get(url)
            .send()
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .text()
            .map_err(|e| e.to_string())?;

        Ok(result)
    }

    pub fn run_xml<T: DeserializeOwned>(&self) -> Result<T, String> {
        let result = Self::run_string(&self)?;
        quick_xml::de::from_str(&result).map_err(|e| e.to_string())
    }

    pub fn run_post<T: DeserializeOwned>(&self, data: String) -> Result<T, String> {
        let client = build_client()?;

        let mut url: Url = Url::parse(&self.address).map_err(|e| e.to_string())?;
        url.set_path(&self.path);
        url.query_pairs_mut().extend_pairs(&self.args.value);

        let result: T = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(data)
            .send()
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .json()
            .map_err(|e| e.to_string())?;

        Ok(result)
    }
}

fn build_client() -> Result<Client, String> {
    Client::builder()
        .user_agent(UAGENT)
        .build()
        .map_err(|e| e.to_string())
}
