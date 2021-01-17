use crate::error::Error;
use crate::gateway::{Gateway, Message};
use crate::heartbeat::Heartbeat;
use crate::http::Http;
use crate::identify::Identify;
use crate::listeners::Listeners;
use crate::payload::{Op, Payload};
use crate::retry::retry;
use log::warn;
use serde_json::Value;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use url::Url;

const V8_URL: &str = "https://discord.com/api/v8";

#[derive(Clone)]
pub struct SmallD {
    http: Arc<Http>,
    gateway: Arc<Gateway>,
    listeners: Arc<Mutex<Listeners>>,
}

impl SmallD {
    pub fn new() -> Result<SmallD, Error> {
        SmallDBuilder::new().build()
    }

    pub fn on_gateway_payload<F>(&self, f: F)
    where
        F: FnMut(&SmallD, &Payload) + Send + Sync + 'static,
    {
        let mut guard = self.listeners.lock().unwrap();
        guard.add(f);
    }

    pub fn on_event<F>(&self, name: &'static str, mut f: F)
    where
        F: FnMut(&SmallD, &Value) + Send + Sync + 'static,
    {
        self.on_gateway_payload(move |s, p| match p {
            Payload {
                op: Op::Dispatch,
                t: Some(event_name),
                d: Some(d),
                ..
            } if *event_name == name => f(s, d),
            _ => (),
        });
    }

    pub fn send_gateway_payload(&self, payload: &Payload) -> Result<(), Error> {
        self.gateway.send(payload)
    }

    pub fn get<S: AsRef<str>>(&self, path: S) -> Result<Value, Error> {
        self.http.get(path)
    }

    pub fn post<S: AsRef<str>>(&self, path: S, json: Value) -> Result<Value, Error> {
        self.http.post(path, json)
    }

    pub fn run(&self) {
        if let Err(err) = retry(Duration::from_millis(5000), || {
            let ws_url = self.get_websocket_url()?;

            self.gateway.connect(ws_url)?;
            loop {
                match self.gateway.read()? {
                    Message::Payload(p) => {
                        let mut guard = self.listeners.lock().unwrap();
                        guard.notify(self, &p);
                    }
                    Message::Close { code, reason } => {
                        break Err(Error::WebSocketClosed { code, reason })
                    }
                    Message::None => sleep(Duration::from_millis(100)),
                }
            }
        }) {
            warn!("Error running Smalld: {}", err);
        }
    }

    pub fn reconnect(&self) {
        self.gateway.close(4900, "Reconnecting...");
    }

    fn get_websocket_url(&self) -> Result<Url, Error> {
        let ws_url_str = self
            .get("/gateway/bot")?
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::illegal_state("Could not get web socket url"))?
            .to_owned();

        Url::parse(&ws_url_str)
            .map_err(|_e| Error::IllegalArgumentError(format!("Bad websocket url: {}", ws_url_str)))
    }
}

pub struct SmallDBuilder {
    token: Option<String>,
    base_url: String,
}

impl SmallDBuilder {
    fn new() -> SmallDBuilder {
        SmallDBuilder {
            token: None,
            base_url: V8_URL.to_string(),
        }
    }

    pub fn token<S: Into<String>>(&mut self, s: S) -> &Self {
        self.token = Some(s.into());
        self
    }

    pub fn base_url<S: Into<String>>(&mut self, s: S) -> &Self {
        self.base_url = s.into();
        self
    }

    fn parse_base_url<S: AsRef<str>>(s: S) -> Result<Url, Error> {
        let error = || {
            Err(Error::ConfigurationError(format!(
                "Bad base_url: {}",
                s.as_ref()
            )))
        };

        match Url::parse(s.as_ref()) {
            Ok(url) if url.cannot_be_a_base() => error(),
            Err(_) => error(),
            Ok(url) => Ok(url),
        }
    }

    fn token_from_env() -> Option<String> {
        match env::var("SMALLD_TOKEN") {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }

    pub fn build(&self) -> Result<SmallD, Error> {
        let token = self
            .token
            .clone()
            .or_else(SmallDBuilder::token_from_env)
            .ok_or_else(|| Error::ConfigurationError("No Discord token provided".to_string()))?;

        let base_url = SmallDBuilder::parse_base_url(&self.base_url)?;

        let smalld: SmallD = SmallD {
            http: Arc::new(Http::new(token.clone(), base_url)),
            gateway: Arc::new(Gateway::new()),
            listeners: Arc::new(Mutex::new(Listeners::new())),
        };

        Heartbeat::new().attach(&smalld);
        Identify::new(token).attach(&smalld);

        Ok(smalld)
    }
}
