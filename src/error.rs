#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Generic error '{0}'.")]
    Generic(String),
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Generic(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Generic(s)
    }
}
