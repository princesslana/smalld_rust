use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug)]
pub struct Payload {
    pub op: u8,
    pub d: Option<Value>,
    pub t: Option<String>,
    pub s: Option<u64>,
}

impl Payload {
    pub fn op(op: u8) -> Payload {
        Payload {
            op: op,
            d: None,
            s: None,
            t: None,
        }
    }

    pub fn d(&mut self, value: Value) -> &mut Self {
        self.d = Some(value);
        self
    }

    pub fn t<S: Into<String>>(&mut self, t: S) -> &mut Self {
        self.t = Some(t.into());
        self
    }

    pub fn s(&mut self, s: u64) -> &mut Self {
        self.s = Some(s);
        self
    }
}