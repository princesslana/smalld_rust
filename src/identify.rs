use log::warn;
use serde_json::json;
use std::env;

use crate::{Op, Payload, PayloadListener, SmallD};

pub struct Identify {
    smalld: SmallD,
}

impl Identify {
    pub fn new(smalld: SmallD) -> Identify {
        Identify { smalld }
    }

    fn identify(&self) {
        let d = json!({ "token": self.smalld.token,
        "properties": {
            "$os": env::consts::OS,
            "$browser": "smalld_rust",
            "$device": "smalld_rust"
        }});

        if let Err(err) = self
            .smalld
            .send_gateway_payload(Payload::op(Op::Identify).d(d))
        {
            warn!("Error sending identify payload: {}", err);
        }
    }
}

impl PayloadListener for Identify {
    fn on_gateway_payload(&self, p: &Payload) {
        match p {
            Payload { op: Op::Hello, .. } => self.identify(),
            _ => (),
        }
    }
}
