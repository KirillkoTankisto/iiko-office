//! Ошибки

#[derive(Debug, thiserror::Error)]
/// Суперструктура с всеми ошибками
pub enum ClientError {
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Xml(#[from] quick_xml::DeError),
    #[error("Unauthorized Access")]
    Unauthorized,
}
