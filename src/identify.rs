use log::warn;
use serde_json::json;
use std::env;

use crate::{Event, Payload, SmallD};

pub struct Identify {}

impl Identify {
    pub fn attach(smalld: &mut SmallD) {
        let identify: Identify = Identify {};

        smalld.on_gateway_payload(move |p| identify.on_gateway_payload(p));
    }

    fn on_gateway_payload(&self, evt: &Event) {
        match evt.payload {
            Payload { op: 10, .. } => self.identify(evt.smalld),
            _ => (),
        }
    }

    fn identify(&self, smalld: &SmallD) {
        if let Err(err) = smalld.send_gateway_payload(Payload {
            op: 2,
            d: Some(json!({ "token": smalld.token,
            "properties": {
                "$os": env::consts::OS,
                "$browser": "smalld_rust",
                "$device": "smalld_rust"
            }})),
            s: None,
            t: None,
        }) {
            warn!("Error sending identify payload: {}", err);
        }
    }
}
