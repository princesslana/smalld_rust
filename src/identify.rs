use log::warn;
use serde_json::json;
use std::env;

use crate::{Op, Payload, SmallD};

pub struct Identify {
    token: String,
}

impl Identify {
    pub fn attach<S: Into<String>>(smalld: &mut SmallD, token: S) {
        let identify = Identify {
            token: token.into(),
        };
        let smalld_clone = smalld.clone();

        smalld.on_gateway_payload(move |p| identify.on_gateway_payload(&smalld_clone, p));
    }

    fn on_gateway_payload(&self, smalld: &SmallD, p: &Payload) {
        match p {
            Payload { op: Op::Hello, .. } => self.identify(&smalld),
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
