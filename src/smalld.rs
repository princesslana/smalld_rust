use serde_json::Value;
use std::env;
use thiserror::Error;
use ureq::{Agent, AgentBuilder};
use url::Url;

const V8_URL: &str = "https://discord.com/api/v8";

pub struct SmallD {
    token: String,
    base_url: Url,
    http: Agent,
}

#[derive(Error, Debug)]
#[error("{0}")]
pub enum Error {
    ConfigurationError(String),
    IllegalArgumentError(String),
    HttpError(#[from] ureq::Error),
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
        })
    }

    fn get<S: AsRef<str>>(&self, path: S) -> Result<Value, Error> {
        let mut url: Url = self.base_url.clone();

        url.path_segments_mut()
            .map_err(|_e| Error::IllegalArgumentError(format!("Bad path: {}", path.as_ref())))?
            .pop_if_empty()
            .extend(path.as_ref().split('/'));

        self.http
            .get(url.as_str())
            .set("Authorization", &format!("Bot {}", self.token))
            .call()?
            .into_json()
            .map_err(|e| e.into())
    }

    pub fn run(&self) {
        self.get("/gateway/bot");
    }
}
