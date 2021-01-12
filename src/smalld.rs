use log::debug;
use retry::delay::Fixed;
use retry::retry;
use serde_json::Value;
use std::env;
use thiserror::Error;
use tungstenite::{connect, Message};
use ureq::{Agent, AgentBuilder};
use url::Url;

const V8_URL: &str = "https://discord.com/api/v8";

pub type Listener = dyn Fn(&Value) -> () + 'static;

pub struct SmallD {
    token: String,
    base_url: Url,
    http: Agent,
    listeners: Vec<Box<Listener>>,
}

#[derive(Error, Debug)]
#[error("{0}")]
pub enum Error {
    ConfigurationError(String),
    IllegalArgumentError(String),
    IllegalStateError(String),
    HttpError(#[from] ureq::Error),
    WebSocketError(#[from] tungstenite::Error),
    IOError(#[from] std::io::Error),
}

impl SmallD {
    pub fn new() -> Result<SmallD, Error> {
        let base_url: Url = Url::parse(V8_URL)
            .map_err(|_e| Error::ConfigurationError(format!("Bad base url: {}", V8_URL)))?;

        if base_url.cannot_be_a_base() {
            return Err(Error::ConfigurationError(format!(
                "Bad base url: {}",
                base_url
            )));
        }

        let token: String = env::var("SMALLD_TOKEN")
            .map_err(|_e| Error::ConfigurationError("Could not find Discord token".to_string()))?;

        Ok(SmallD {
            token: token,
            base_url: base_url,
            http: AgentBuilder::new().build(),
            listeners: vec![],
        })
    }

    pub fn on_gateway_payload<F>(&mut self, f: F)
    where
        F: Fn(&Value) -> () + 'static,
    {
        self.listeners.push(Box::new(f));
    }

    pub fn get<S: AsRef<str>>(&self, path: S) -> Result<Value, Error> {
        let mut url: Url = self.base_url.clone();

        url.path_segments_mut()
            .map_err(|_e| Error::IllegalArgumentError(format!("Bad path: {}", path.as_ref())))?
            .pop_if_empty()
            .extend(path.as_ref().trim_start_matches('/').split('/'));

        self.http
            .get(url.as_str())
            .set("Authorization", &format!("Bot {}", self.token))
            .call()?
            .into_json()
            .map_err(|e| e.into())
    }

    pub fn run(&self) {
        retry(Fixed::from_millis(5000), || {
            let ws_url = self
                .get("/gateway/bot")?
                .get("url")
                .and_then(Value::as_str)
                .ok_or(Error::IllegalStateError(
                    "Could not get web socket url".to_string(),
                ))?
                .to_owned();

            let (mut socket, _) = connect(ws_url)?;
            loop {
                match socket.read_message()? {
                    Message::Text(s) => {
                        debug!("Payload received: {}", s);
                        if let Ok(json) = serde_json::from_str(&s) {
                            for l in self.listeners.iter() {
                                l(&json)
                            }
                        }
                    }
                    Message::Close(_) => break,
                    Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => {}
                }
            }
            Ok::<(), Error>(())
        });
    }
}
