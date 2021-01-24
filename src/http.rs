use crate::error::Error;
use serde_json::{json, Value};
use ureq::{Agent, AgentBuilder, Request, Response};
use url::Url;

pub(crate) struct Http {
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

    fn build_url(&self, path: &str) -> Result<Url, Error> {
        let mut url: Url = self.base_url.clone();

        url.path_segments_mut()
            .map_err(|_e| Error::IllegalArgumentError(format!("Bad path: {}", path)))?
            .pop_if_empty()
            .extend(path.trim_start_matches('/').split('/'));

        Ok(url)
    }

    pub(crate) fn with_request<P, F>(
        &self,
        method: &str,
        path: P,
        params: &[(String, String)],
        f: F,
    ) -> Result<Value, Error>
    where
        P: AsRef<str>,
        F: FnOnce(Request) -> Result<Response, ureq::Error>,
    {
        let mut request = self
            .agent
            .request_url(method, &self.build_url(path.as_ref())?)
            .set("Authorization", &self.authorization)
            .set("User-Agent", &self.user_agent);

        for (k, v) in params.iter() {
            request = request.query(k, v);
        }

        let response = f(request)?;

        match response.status() {
            204 => Ok(json!({})),
            _ => response.into_json().map_err(|e| e.into()),
        }
    }
}
