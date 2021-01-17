use thiserror::Error;

#[derive(Error, Debug)]
#[error("{0}")]
pub enum Error {
    ConfigurationError(String),
    IllegalArgumentError(String),
    IllegalStateError(String),
    HttpError(Box<ureq::Error>),
    WebSocketError(#[from] tungstenite::Error),
    IOError(#[from] std::io::Error),
}

impl From<ureq::Error> for Error {
    fn from(error: ureq::Error) -> Self {
        Error::HttpError(Box::new(error))
    }
}

impl Error {
    pub fn illegal_state<S: Into<String>>(msg: S) -> Error {
        Error::IllegalStateError(msg.into())
    }
}
