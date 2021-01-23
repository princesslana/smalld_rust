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

    pub fn get<S: AsRef<str>>(&self, path: S, params: QueryParameters) -> Result<Value, Error> {
        self.with_request("GET", path.as_ref(), params, |r| r.call())
    }

    pub fn post<S: AsRef<str>>(
        &self,
        path: S,
        params: QueryParameters,
        json: Value,
    ) -> Result<Value, Error> {
        self.with_request("POST", path.as_ref(), params, |r| r.send_json(json))
    }

    pub fn put<S: AsRef<str>>(
        &self,
        path: S,
        params: QueryParameters,
        json: Value,
    ) -> Result<Value, Error> {
        self.with_request("PUT", path.as_ref(), params, |r| r.send_json(json))
    }

    pub fn patch<S: AsRef<str>>(&self, path: S, json: Value) -> Result<Value, Error> {
        self.with_request("PATCH", path.as_ref(), QueryParameters::new(), |r| {
            r.send_json(json)
        })
    }

    pub fn delete<S: AsRef<str>>(&self, path: S) -> Result<Value, Error> {
        self.with_request("DELETE", path.as_ref(), QueryParameters::new(), |r| {
            r.call()
        })
    }

    fn build_url(&self, path: &str) -> Result<Url, Error> {
        let mut url: Url = self.base_url.clone();

        url.path_segments_mut()
            .map_err(|_e| Error::IllegalArgumentError(format!("Bad path: {}", path)))?
            .pop_if_empty()
            .extend(path.trim_start_matches('/').split('/'));

        Ok(url)
    }

    fn with_request<F>(
        &self,
        method: &str,
        path: &str,
        params: QueryParameters,
        f: F,
    ) -> Result<Value, Error>
    where
        F: FnOnce(Request) -> Result<Response, ureq::Error>,
    {
        let mut request = self
            .agent
            .request_url(method, &self.build_url(path)?)
            .set("Authorization", &self.authorization)
            .set("User-Agent", &self.user_agent);

        request = params.apply_to(request);

        let response = f(request)?;

        match response.status() {
            204 => Ok(json!({})),
            _ => response.into_json().map_err(|e| e.into()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct QueryParameters(Vec<(String, String)>);

/// Struct for holding query parameters to be added to a url with a HTTP request.
/// Parameters may be added using the add method:
///
/// ```no_run
/// use smalld::QueryParameters;
///
/// let params = QueryParameters::new()
///     .add("key1", "value1")
///     .add("key2", "value2");
/// ```
///
impl QueryParameters {
    pub fn new() -> Self {
        QueryParameters(Vec::new())
    }

    pub fn add<A, B>(mut self, key: A, value: B) -> Self
    where
        A: Into<String>,
        B: Into<String>,
    {
        self.0.push((key.into(), value.into()));
        self
    }

    pub(self) fn apply_to(&self, req_in: Request) -> Request {
        let mut req_out = req_in;
        for (k, v) in self.0.iter() {
            req_out = req_out.query(k, v);
        }
        req_out
    }
}

impl Default for QueryParameters {
    fn default() -> Self {
        Self::new()
    }
}
