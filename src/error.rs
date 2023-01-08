type BracketError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Generic error '{0}'.")]
    Generic(String),
    #[error(transparent)]
    Other(#[from] BracketError),
}
