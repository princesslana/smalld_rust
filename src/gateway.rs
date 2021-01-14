use log::debug;
use std::io::ErrorKind;
use std::sync::Mutex;
use tungstenite::client::AutoStream;
use tungstenite::stream::Stream;
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
    None,
}

impl Gateway {
    pub fn new() -> Gateway {
        Gateway {
            web_socket: Mutex::new(None),
        }
    }

    pub fn connect(&self, url: Url) -> Result<(), Error> {
        let (mut socket, _) = connect(url.as_str())?;

        match socket.get_mut() {
            Stream::Plain(s) => s.set_nonblocking(true),
            Stream::Tls(s) => s.get_mut().set_nonblocking(true),
        }?;
        //socket.get_mut().set_nonblocking();

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
            .ok_or_else(|| Error::IllegalStateError("No gateway connected".to_string()))?;

        f(ws).map_err(|e| e.into())
    }

    pub fn send(&self, payload: &Payload) -> Result<(), Error> {
        let txt: String = serde_json::to_string(payload).map_err(|_e| {
            Error::IllegalArgumentError(format!("Unable to convert payload to json {:?}", payload))
        })?;

        debug!("Send >>> {}", txt);

        let txt_ref: &str = txt.as_ref();
        self.with_web_socket(|ws| ws.write_message(WsMessage::text(txt_ref)))
    }

    pub fn read(&self) -> Result<Message, Error> {
        let ws_msg = self.with_web_socket(|ws| ws.read_message());

        match ws_msg {
            Ok(WsMessage::Text(s)) => {
                debug!("Recv <<< {}", s);
                let payload = serde_json::from_str(&s).map_err(|_e| {
                    Error::IllegalStateError(format!("Bad payload received from gateway: {}", s))
                })?;
                Ok(Message::Payload(payload))
            }
            Ok(WsMessage::Close(why)) => {
                debug!("Close !!! {:?}", why);
                self.close();

                Ok(why.map_or(
                    Message::Close {
                        code: None,
                        reason: "Unknown".to_string(),
                    },
                    |c| Message::Close {
                        code: Some(c.code.into()),
                        reason: c.reason.to_string(),
                    },
                ))
            }
            Ok(_) => Ok(Message::None),
            Err(Error::WebSocketError(tungstenite::Error::Io(err)))
                if err.kind() == ErrorKind::WouldBlock =>
            {
                Ok(Message::None)
            }
            Err(err) => Err(err),
        }
    }
}

impl Drop for Gateway {
    fn drop(&mut self) {
        self.close()
    }
}
