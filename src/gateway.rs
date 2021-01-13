use log::debug;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Mutex;
use tungstenite::client::AutoStream;
use tungstenite::{connect, Message, WebSocket};
use url::Url;

use crate::smalld::Error;

type WS = WebSocket<AutoStream>;

pub struct Gateway {
    web_socket: Mutex<Option<WS>>,
}

#[derive(Deserialize, Debug)]
pub struct Payload {
    op: u8,
    d: Option<Value>,
    t: Option<String>,
    s: Option<u64>,
}

impl Gateway {
    pub fn new() -> Gateway {
        Gateway {
            web_socket: Mutex::new(None),
        }
    }

    pub fn connect(&self, url: Url) -> Result<(), Error> {
        let (socket, _) = connect(url.as_str())?;

        let mut lock = self.web_socket.lock().unwrap();
        *lock = Some(socket);

        Ok(())
    }

    pub fn close(&self) {
        let mut lock = self.web_socket.lock().unwrap();
        *lock = None
    }

    fn with_web_socket<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: Fn(&mut WS) -> Result<R, tungstenite::Error>,
    {
        let mut lock = self.web_socket.lock().unwrap();

        let ws = lock
            .as_mut()
            .ok_or(Error::IllegalStateError("No gateway connected".to_string()))?;

        f(ws).map_err(|e| e.into())
    }

    pub fn send<S: AsRef<str>>(&self, payload: S) -> Result<(), Error> {
        self.with_web_socket(|ws| {
            let txt: String = payload.as_ref().to_string();
            debug!("Sending to gateway: {}", txt);
            ws.write_message(Message::Text(txt))
        })
    }

    pub fn read(&self) -> Result<Message, Error> {
        self.with_web_socket(|ws| ws.read_message())
    }
}

impl Drop for Gateway {
    fn drop(&mut self) {
        self.close()
    }
}
