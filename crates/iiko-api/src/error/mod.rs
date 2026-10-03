//! Ошибки

#[derive(Debug, thiserror::Error)]
/// Суперструктура с всеми ошибками
pub enum ClientError {
    #[error(transparent)]
    Http(#[from] ureq::Error),
    #[error(transparent)]
    Uri(#[from] ureq::http::uri::InvalidUri),
    #[error("Invalid Address")]
    Address,
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Xml(#[from] quick_xml::DeError),
    #[error("Unauthorized Access")]
    Unauthorized,
    #[error("Access Denied")]
    Forbidden,
    #[error("Server returned HTTP {0}")]
    Status(u16)
}
