use crate::{Op, Payload, SmallD};
use log::warn;
use serde_json::json;
use std::sync::{Arc, Mutex, Once};
use std::thread::{sleep, spawn};
use std::time::Duration;

#[derive(Clone)]
pub struct Heartbeat {
    interval: Arc<Mutex<Option<u64>>>,
    sequence_number: Arc<Mutex<Option<u64>>>,
    thread: Arc<Once>,
}

impl Heartbeat {
    pub fn new() -> Self {
        Heartbeat {
            interval: Arc::new(Mutex::new(None)),
            sequence_number: Arc::new(Mutex::new(None)),
            thread: Arc::new(Once::new()),
        }
    }

    pub fn attach(self, smalld: &SmallD) {
        smalld.on_gateway_payload(move |s, p| self.on_gateway_payload(s, p));
    }

    fn on_gateway_payload(&self, smalld: &SmallD, p: &Payload) {
        match p {
            Payload {
                op: Op::Hello,
                d: Some(d),
                ..
            } => {
                if let Some(interval) = d.get("heartbeat_interval").and_then(|v| v.as_u64()) {
                    self.set_interval(interval);

                    self.thread.call_once(|| {
                        let smalld = smalld.clone();
                        let heartbeat = self.clone();
                        spawn(move || heartbeat.run(&smalld));
                    });
                }
            }
            Payload { s: Some(s), .. } => self.set_sequence_number(*s),
            _ => (),
        }
    }

    fn interval(&self) -> Option<u64> {
        *self.interval.lock().unwrap()
    }

    fn set_interval(&self, interval: u64) {
        let mut lock = self.interval.lock().unwrap();
        *lock = Some(interval);
    }

    fn sequence_number(&self) -> Option<u64> {
        *self.sequence_number.lock().unwrap()
    }

    fn set_sequence_number(&self, sequence_number: u64) {
        let mut lock = self.sequence_number.lock().unwrap();
        *lock = Some(sequence_number);
    }

    fn run(&self, smalld: &SmallD) {
        loop {
            match self.interval() {
                None => {
                    sleep(Duration::from_secs(5));
                    continue;
                }
                Some(ms) => {
                    sleep(Duration::from_millis(ms));
                }
            }

            self.send(&smalld);
        }
    }

    fn send(&self, smalld: &SmallD) {
        let d = self.sequence_number().map_or(json!(null), |n| json!(n));

        if let Err(err) = smalld.send_gateway_payload(Payload::op(Op::Heartbeat).d(d)) {
            warn!("Error sending heartbeat: {}", err);
        }
    }
}
