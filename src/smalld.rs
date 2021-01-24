use crate::error::Error;
use crate::gateway::{Gateway, Message};
use crate::heartbeat::Heartbeat;
use crate::http::Http;
use crate::identify::Identify;
use crate::intents::Intent;
use crate::listeners::Listeners;
use crate::payload::{Op, Payload};
use crate::resource::Resource;
use crate::retry::retry;
use log::warn;
use serde_json::Value;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use url::Url;

const V8_URL: &str = "https://discord.com/api/v8";

/// SmallD is the central point for access to the Discord API.
///
/// Methods can be split into three categories:
///   * **Lifecycle**  
///     The methods for creating, running, and closing the connection with
///     Discord. These methods are [`new`](SmallD#function.new), [`run`](SmallD#function.run), and
///     [`reconnect`](SmallD#function.reconnect)
///
///   * **Gateway**  
///     The methods for communicating with the Discord gateway. Receiving is handled via
///     [`on_gateway_payload`](SmallD#on_gateway_payload) and [`on_event`](SmallD#on_event) and
///     sending is via [`send_gateway_payload`](SmallD#send_gateway_payload)
///
///   * **Resources**
///     The method for acessing Discord's rest based resource apis. This is the
///     [`resource`](SmallD#function.resource) method, which provides a builder like interface to
///     access Discord resources.
///
#[derive(Clone)]
pub struct SmallD {
    http: Arc<Http>,
    gateway: Arc<Gateway>,
    listeners: Arc<Mutex<Listeners>>,
}

impl SmallD {
    /// Equivalent to [`SmallDBuilder::new().build()`](SmallDBuilder#new).
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

    pub fn resource<S: Into<String>>(&self, path: S) -> Resource {
        Resource::new(self.http.clone(), path)
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
            .resource("/gateway/bot")
            .get()?
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::illegal_state("Could not get websocket url"))?
            .to_owned();

        Url::parse(&ws_url_str)
            .map_err(|_e| Error::IllegalArgumentError(format!("Bad websocket url: {}", ws_url_str)))
    }
}

/// Builder to configure and create a [`SmallD`](SmallD).
pub struct SmallDBuilder {
    token: Option<String>,
    base_url: String,
    intents: u16,
}

impl SmallDBuilder {
    /// Creates a [`SmallDBuilder`](SmallDBuilder) configured with useful defaults.
    /// This includes a token retrieved from the environment variable `SMALLD_TOKEN`,
    /// all unprivileged [gateway
    /// intents](https://discord.com/developers/docs/topics/gateway#gateway-intents),
    /// and to use [v8](https://discord.com/developers/docs/reference#api-versioning) of the
    /// Discord API.
    pub fn new() -> Self {
        Self {
            token: None,
            base_url: V8_URL.to_string(),
            intents: Intent::UNPRIVILEGED,
        }
    }

    pub fn token<S: Into<String>>(&mut self, s: S) -> &mut Self {
        self.token = Some(s.into());
        self
    }

    pub fn base_url<S: Into<String>>(&mut self, s: S) -> &mut Self {
        self.base_url = s.into();
        self
    }

    pub fn intents<M: Into<u16>>(&mut self, intents: M) -> &mut Self {
        self.intents = intents.into();
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
            .or_else(Self::token_from_env)
            .ok_or_else(|| Error::ConfigurationError("No Discord token provided".to_string()))?;

        let base_url = Self::parse_base_url(&self.base_url)?;

        let smalld: SmallD = SmallD {
            http: Arc::new(Http::new(token.clone(), base_url)),
            gateway: Arc::new(Gateway::new()),
            listeners: Arc::new(Mutex::new(Listeners::new())),
        };

        Heartbeat::new().attach(&smalld);
        Identify::new(token, self.intents).attach(&smalld);

        Ok(smalld)
    }
}

impl Default for SmallDBuilder {
    fn default() -> Self {
        Self::new()
    }
}
