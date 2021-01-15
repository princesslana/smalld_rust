use crate::error::Error;
use crate::gateway::{Gateway, Message};
use crate::heartbeat::Heartbeat;
use crate::http::Http;
use crate::identify::Identify;
use crate::listeners::Listeners;
use crate::payload::{Op, Payload};
use log::warn;
use retry::delay::Fixed;
use retry::retry;
use serde_json::Value;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use url::Url;

const V8_URL: &str = "https://discord.com/api/v8";

#[derive(Clone)]
pub struct SmallD {
    config: Config,
    http: Arc<Http>,
    gateway: Arc<Gateway>,
    listeners: Arc<Mutex<Listeners<Payload>>>,
}

#[derive(Clone)]
pub struct Config {
    pub token: String,
    pub base_url: Url,
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

        let config = Config { token, base_url };
        let config_clone = config.clone();

        let mut smalld: SmallD = SmallD {
            config: config_clone,
            http: Arc::new(Http::new(&config)),
            gateway: Arc::new(Gateway::new()),
            listeners: Arc::new(Mutex::new(Listeners::new())),
        };

        Heartbeat::attach(&mut smalld);
        Identify::attach(&mut smalld);

        Ok(smalld)
    }

    pub fn token(&self) -> &String {
        &self.config.token
    }

    pub fn on_gateway_payload<F>(&mut self, f: F)
    where
        F: Fn(&Payload) + Send + Sync + 'static,
    {
        let mut guard = self.listeners.lock().unwrap();
        guard.add(f);
    }

    pub fn on_event<F>(&mut self, name: &'static str, f: F)
    where
        F: Fn(&Value) + Send + Sync + 'static,
    {
        self.on_gateway_payload(move |p| match p {
            Payload {
                op: Op::Dispatch,
                t: Some(event_name),
                d: Some(d),
                ..
            } if *event_name == name => f(d),
            _ => (),
        });
    }

    pub fn send_gateway_payload(&self, payload: &Payload) -> Result<(), Error> {
        self.gateway.send(payload)
    }

    pub fn get<S: AsRef<str>>(&self, path: S) -> Result<Value, Error> {
        self.http.get(path)
    }

    pub fn run(&self) {
        if let Err(err) = retry(Fixed::from_millis(5000), || {
            let ws_url_str = self
                .get("/gateway/bot")?
                .get("url")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    Error::IllegalStateError("Could not get web socket url".to_string())
                })?
                .to_owned();

            let ws_url = Url::parse(&ws_url_str).map_err(|_e| {
                Error::IllegalArgumentError(format!("Bad websocket url: {}", ws_url_str))
            })?;

            self.gateway.connect(ws_url)?;
            loop {
                match self.gateway.read()? {
                    Message::Payload(p) => {
                        let guard = self.listeners.lock().unwrap();
                        guard.notify(p);
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
