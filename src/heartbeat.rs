use crate::{Payload, SmallD};
use log::warn;
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Duration;

pub struct Heartbeat {
    heartbeat_interval: Option<u64>,
    sequence_number: Option<u64>,
}

impl Heartbeat {
    pub fn attach(smalld: &mut SmallD) {
        let heartbeat: Arc<Mutex<Heartbeat>> = Arc::new(Mutex::new(Heartbeat {
            heartbeat_interval: None,
            sequence_number: None,
        }));

        {
            let heartbeat = Arc::clone(&heartbeat);
            let smalld = smalld.clone();
            spawn(move || Heartbeat::run(&smalld, heartbeat));
        }

        smalld.on_gateway_payload(move |p| {
            let mut lock = heartbeat.lock().unwrap();
            lock.on_gateway_payload(p)
        });
    }

    fn run(smalld: &SmallD, heartbeat: Arc<Mutex<Heartbeat>>) {
        loop {
            let heartbeat_interval = {
                let lock = heartbeat.lock().unwrap();
                lock.heartbeat_interval
            };

            match heartbeat_interval {
                None => {
                    sleep(Duration::from_secs(5));
                    continue;
                }
                Some(ms) => {
                    sleep(Duration::from_millis(ms));
                }
            }

            {
                let lock = heartbeat.lock().unwrap();
                lock.send(smalld);
            }
        }
    }

    fn send(&self, smalld: &SmallD) {
        let d = self.sequence_number.map_or(json!(null), |n| json!(n));

        if let Err(err) = smalld.send_gateway_payload(Payload::op(1).d(d)) {
            warn!("Error sending heartbeat: {}", err);
        }
    }

    fn on_gateway_payload(&mut self, p: &Payload) {
        match p {
            Payload {
                op: 10, d: Some(d), ..
            } => self.heartbeat_interval = d.get("heartbeat_interval").and_then(|v| v.as_u64()),
            Payload { s: Some(s), .. } => self.sequence_number = Some(*s),
            _ => (),
        }
    }
}
