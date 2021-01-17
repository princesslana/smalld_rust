use log::warn;
use serde_json::json;
use std::env;

use crate::{Op, Payload, SmallD};

pub struct Identify {
    token: String,
}

impl Identify {
    pub fn new<S: Into<String>>(token: S) -> Self {
        Identify {
            token: token.into(),
        }
    }

    pub fn attach(self, smalld: &SmallD) {
        smalld.on_gateway_payload(move |s, p| self.on_gateway_payload(s, p));
    }

    fn on_gateway_payload(&self, smalld: &SmallD, p: &Payload) {
        match p {
            Payload { op: Op::Hello, .. } => self.identify(&smalld),
            Payload {
                op: Op::Reconnect, ..
            } => smalld.reconnect(),
            _ => (),
        }
    }

    fn identify(&self, smalld: &SmallD) {
        let d = json!({ "token": self.token,
        "properties": {
            "$os": env::consts::OS,
            "$browser": "smalld_rust",
            "$device": "smalld_rust"
        }});

        if let Err(err) = smalld.send_gateway_payload(Payload::op(Op::Identify).d(d)) {
            warn!("Error sending identify payload: {}", err);
        }
    }
}
