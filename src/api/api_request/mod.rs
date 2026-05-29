use reqwest::{Url, blocking::Client};
use serde::de::DeserializeOwned;
use std::error::Error;

const UAGENT: &'static str = "iiko-office-free/0.1";

pub struct ApiArgs<const N: usize> {
    value: [(String, String); N],
}

impl<const N: usize> ApiArgs<N> {
    pub fn new<S: Into<String>>(input: [(S, S); N]) -> Self {
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
    pub fn new<S: Into<String>>(address: S, path: S, args: ApiArgs<N>) -> Self {
        Self {
            address: address.into(),
            path: path.into(),
            args,
        }
    }

    pub fn run<T: DeserializeOwned>(&self) -> Result<T, Box<dyn Error>> {
        let client = build_client()?;

        let mut url: Url = Url::parse(&self.address)?;
        url.set_path(&self.path);

        let result: T = client.get(url).send()?.json()?;

        Ok(result)
    }

    pub fn run_string(&self) -> Result<String, Box<dyn Error>> {
        let client = build_client()?;

        let mut url: Url = Url::parse(&self.address)?;
        url.set_path(&self.path);
        url.query_pairs_mut().extend_pairs(&self.args.value);

        let result: String = client.get(url).send()?.text()?;

        Ok(result)
    }
}

fn build_client() -> Result<Client, Box<dyn Error>> {
    Ok(Client::builder().user_agent(UAGENT).build()?)
}
