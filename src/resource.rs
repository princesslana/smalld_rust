use crate::error::Error;
use crate::http::Http;
use serde_json::Value;
use std::sync::Arc;

pub struct Resource {
    http: Arc<Http>,
    path: String,
    parameters: Vec<(String, String)>,
}

impl Resource {
    pub(crate) fn new<S: Into<String>>(http: Arc<Http>, path: S) -> Resource {
        Resource {
            http,
            path: path.into(),
            parameters: Vec::new(),
        }
    }

    pub fn query<K, V>(mut self, key: K, value: V) -> Resource
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.parameters.push((key.into(), value.into()));
        self
    }

    pub fn get(self) -> Result<Value, Error> {
        self.call("GET")
    }

    pub fn post(self, json: Value) -> Result<Value, Error> {
        self.send_json("POST", json)
    }

    pub fn put(self, json: Value) -> Result<Value, Error> {
        self.send_json("PUT", json)
    }

    pub fn patch(self, json: Value) -> Result<Value, Error> {
        self.send_json("PATCH", json)
    }

    pub fn delete(self) -> Result<Value, Error> {
        self.call("DELETE")
    }

    fn call(self, method: &str) -> Result<Value, Error> {
        self.http
            .with_request(method, self.path, &self.parameters, |r| r.call())
    }

    fn send_json(self, method: &str, json: Value) -> Result<Value, Error> {
        self.http
            .with_request(method, self.path, &self.parameters, |r| r.send_json(json))
    }
}
