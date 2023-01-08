#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Generic error '{0}'.")]
    Generic(String),
}
