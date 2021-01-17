use crate::retry::RetryableError;
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

    #[error("{code:?}: {reason}")]
    WebSocketClosed {
        code: Option<u16>,
        reason: String,
    },
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

const FATAL_WEBSOCKET_CODES: [u16; 6] = [4004, 4010, 4011, 4012, 4013, 4014];

impl RetryableError for Error {
    fn is_fatal(&self) -> bool {
        matches!(self,
            Error::WebSocketClosed {
                code: Some(code), ..
            } if FATAL_WEBSOCKET_CODES.contains(code)
        )
    }
}
