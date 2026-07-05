#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Api(#[from] iiko_api::error::ClientError),
    #[error("Failed to Save Configuration")]
    Config(#[from] std::io::Error),
    #[error("Not Logged in")]
    NotLoggedIn,
    #[error("Internal Error")]
    Internal,
}
