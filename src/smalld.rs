use crate::gateway::{Gateway, Message};
use crate::heartbeat::Heartbeat;
use crate::identify::Identify;
use crate::listeners::Listeners;
use crate::payload::Payload;
use log::warn;
use retry::delay::Fixed;
use retry::retry;
use serde_json::Value;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use thiserror::Error;
use ureq::{Agent, AgentBuilder};
use url::Url;

const V8_URL: &str = "https://discord.com/api/v8";

pub struct Event<'a> {
    pub smalld: &'a SmallD,
    pub payload: Payload,
}

#[derive(Clone)]
pub struct SmallD {
    pub token: String,
    base_url: Url,
    http: Agent,
    gateway: Arc<Gateway>,
    listeners: Arc<Mutex<Listeners>>,
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

        let mut smalld: SmallD = SmallD {
            token: token,
            base_url: base_url,
            http: AgentBuilder::new().build(),
            gateway: Arc::new(Gateway::new()),
            listeners: Arc::new(Mutex::new(Listeners::new())),
        };

        Heartbeat::attach(&mut smalld);
        Identify::attach(&mut smalld);

        Ok(smalld)
    }

    pub fn on_gateway_payload<F>(&mut self, f: F)
    where
        F: Fn(&Event) -> () + Send + Sync + 'static,
    {
        let mut guard = self.listeners.lock().unwrap();
        guard.add(f);
    }

    pub fn send_gateway_payload(&self, payload: &Payload) -> Result<(), Error> {
        self.gateway.send(payload)
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
        if let Err(err) = retry(Fixed::from_millis(5000), || {
            let ws_url_str = self
                .get("/gateway/bot")?
                .get("url")
                .and_then(Value::as_str)
                .ok_or(Error::IllegalStateError(
                    "Could not get web socket url".to_string(),
                ))?
                .to_owned();

            let ws_url = Url::parse(&ws_url_str).map_err(|_e| {
                Error::IllegalArgumentError(format!("Bad websocket url: {}", ws_url_str))
            })?;

            self.gateway.connect(ws_url)?;
            loop {
                match self.gateway.read()? {
                    Message::Payload(p) => {
                        let evt = Event {
                            smalld: &self,
                            payload: p,
                        };
                        let guard = self.listeners.lock().unwrap();
                        guard.notify(&evt);
                    }
                    Message::Close { .. } => break,
                    Message::None => sleep(Duration::from_millis(100)),
                }
            }
            Ok::<(), Error>(())
        }) {
            warn!("Error running Smalld: {}", err);
        }
    }
}
