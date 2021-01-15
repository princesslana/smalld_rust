use crate::error::Error;
use serde_json::Value;
use ureq::{Agent, AgentBuilder};
use url::Url;

pub struct Http {
    token: String,
    base_url: Url,
    agent: Agent,
}

impl Http {
    pub fn new<S: Into<String>>(token: S, base_url: Url) -> Http {
        Http {
            token: token.into(),
            base_url,
            agent: AgentBuilder::new().build(),
        }
    }

    pub fn get<S: AsRef<str>>(&self, path: S) -> Result<Value, Error> {
        let mut url: Url = self.base_url.clone();

        url.path_segments_mut()
            .map_err(|_e| Error::IllegalArgumentError(format!("Bad path: {}", path.as_ref())))?
            .pop_if_empty()
            .extend(path.as_ref().trim_start_matches('/').split('/'));

        self.agent
            .get(url.as_str())
            .set("Authorization", &format!("Bot {}", self.token))
            .call()?
            .into_json()
            .map_err(|e| e.into())
    }
}
