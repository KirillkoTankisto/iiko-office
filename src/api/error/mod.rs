#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error(transparent)]
    Iiko(#[from] IikoError),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Xml(#[from] quick_xml::DeError),
}

#[derive(Debug, thiserror::Error)]
pub enum IikoError {
    #[error("Failed to get UserData (Mutex lock failed)")]
    UdataFailed,
}
