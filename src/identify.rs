use log::info;
use serde_json::{json, Value};
use std::env;
use ureq::Agent;

use crate::{Payload, SmallD};

pub struct Identify {}

impl Identify {
    pub fn attach(smalld: &mut SmallD) {
        let identify: Identify = Identify {};

        smalld.on_gateway_payload(move |p| identify.on_gateway_payload(p));
    }

    fn on_gateway_payload(&self, p: &Payload) {
        info!("payload received in identify! {:?}", p);
    }
    /*
    fn identify(&mut self) {
        self.smalld.send_gateway_payload(json!({
            "op": 2,
            "d": {
                "token": self.smalld.token,
                "properties": {
                    "$os": env::consts::OS,
                    "$browser": "smalld_rust",
                    "$device": "smalld_rust"
                }
            }
        }));
    }
    */
}
