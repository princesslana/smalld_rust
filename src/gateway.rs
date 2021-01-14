use log::debug;
use std::sync::Mutex;
use tungstenite::client::AutoStream;
use tungstenite::{connect, Message as WsMessage, WebSocket};
use url::Url;

use crate::payload::Payload;
use crate::smalld::Error;

type WS = WebSocket<AutoStream>;

pub struct Gateway {
    web_socket: Mutex<Option<WS>>,
}

#[derive(Debug)]
pub enum Message {
    Payload(Payload),
    Close { code: Option<u16>, reason: String },
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

    pub fn send(&self, payload: &Payload) -> Result<(), Error> {
        let txt: String = serde_json::to_string(payload).map_err(|_e| {
            Error::IllegalArgumentError(format!("Unable to convert payload to json {:?}", payload))
        })?;

        debug!("Sending to gateway: {}", txt);

        let txt_ref: &str = txt.as_ref();

        self.with_web_socket(|ws| ws.write_message(WsMessage::text(txt_ref)))
    }

    pub fn read(&self) -> Result<Message, Error> {
        let ws_msg = self.with_web_socket(|ws| ws.read_message())?;

        let msg = match ws_msg {
            WsMessage::Text(s) => {
                let payload = serde_json::from_str(&s).map_err(|_e| {
                    Error::IllegalStateError(format!("Bad payload received from gateway: {}", s))
                })?;
                Message::Payload(payload)
            }
            WsMessage::Close(why) => {
                self.close();

                why.map_or(
                    Message::Close {
                        code: None,
                        reason: "Unknown".to_string(),
                    },
                    |c| Message::Close {
                        code: Some(c.code.into()),
                        reason: c.reason.to_string(),
                    },
                )
            }
            WsMessage::Ping(_) | WsMessage::Pong(_) | WsMessage::Binary(_) => self.read()?,
        };

        debug!("Received from Gatway: {:?}", msg);

        Ok(msg)
    }
}

impl Drop for Gateway {
    fn drop(&mut self) {
        self.close()
    }
}
