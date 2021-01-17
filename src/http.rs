use crate::error::Error;
use serde_json::Value;
use ureq::{Agent, AgentBuilder, Request, Response};
use url::Url;

pub struct Http {
    authorization: String,
    user_agent: String,
    base_url: Url,
    agent: Agent,
}

impl Http {
    pub fn new<S: AsRef<str>>(token: S, base_url: Url) -> Http {
        Http {
            authorization: format!("Bot {}", token.as_ref()),
            user_agent: format!(
                "DiscordBot ({}, {})",
                env!("CARGO_PKG_REPOSITORY"),
                env!("CARGO_PKG_VERSION")
            ),
            base_url,
            agent: AgentBuilder::new().build(),
        }
    }

    pub fn get<S: AsRef<str>>(&self, path: S) -> Result<Value, Error> {
        self.with_request("GET", path.as_ref(), |r| r.call())
    }

    pub fn post<S: AsRef<str>>(&self, path: S, json: Value) -> Result<Value, Error> {
        self.with_request("POST", path.as_ref(), |r| r.send_json(json))
    }

    fn build_url(&self, path: &str) -> Result<Url, Error> {
        let mut url: Url = self.base_url.clone();

        url.path_segments_mut()
            .map_err(|_e| Error::IllegalArgumentError(format!("Bad path: {}", path)))?
            .pop_if_empty()
            .extend(path.trim_start_matches('/').split('/'));

        Ok(url)
    }

    fn with_request<F>(&self, method: &str, path: &str, f: F) -> Result<Value, Error>
    where
        F: FnOnce(Request) -> Result<Response, ureq::Error>,
    {
        let request = self
            .agent
            .request_url(method, &self.build_url(path)?)
            .set("Authorization", &self.authorization)
            .set("User-Agent", &self.user_agent);

        let response = f(request)?;

        response.into_json().map_err(|e| e.into())
    }
}
