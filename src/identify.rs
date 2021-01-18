use crate::{Error, Op, Payload, SmallD};
use log::warn;
use serde_json::json;
use std::env;
use std::thread::sleep;
use std::time::Duration;

pub struct Identify {
    token: String,
    intents_bitmask: u16,
    session_id: Option<String>,
    sequence_number: Option<u64>,
}

impl Identify {
    pub fn new<S: Into<String>>(token: S, intents_bitmask: u16) -> Self {
        Identify {
            token: token.into(),
            intents_bitmask,
            session_id: None,
            sequence_number: None,
        }
    }

    pub fn attach(mut self, smalld: &SmallD) {
        smalld.on_gateway_payload(move |s, p| self.on_gateway_payload(s, p));
    }

    fn on_gateway_payload(&mut self, smalld: &SmallD, p: &Payload) {
        match p {
            Payload { op: Op::Hello, .. } => self
                .try_resume(smalld)
                .unwrap_or_else(|_| self.identify(smalld)),

            Payload {
                op: Op::Reconnect, ..
            } => smalld.reconnect(),

            Payload {
                op: Op::Dispatch,
                t: Some(evt),
                d: Some(d),
                ..
            } if evt == "READY" => {
                self.set_session_id(d.get("session_id").and_then(|v| v.as_str()))
            }

            Payload {
                op: Op::InvalidSession,
                ..
            } => self.on_invalid_session(smalld),

            Payload { s: Some(s), .. } => self.set_sequence_number(*s),

            _ => (),
        }
    }

    fn on_invalid_session(&mut self, smalld: &SmallD) {
        self.set_session_id(None);
        sleep(Duration::from_secs(2));
        self.identify(smalld);
    }

    fn set_session_id(&mut self, session_id: Option<&str>) {
        self.session_id = session_id.map(|s| s.into());
    }

    fn set_sequence_number(&mut self, sequence_number: u64) {
        self.sequence_number = Some(sequence_number);
    }

    fn identify(&self, smalld: &SmallD) {
        let d = json!({
            "token": self.token,
            "properties": {
                "$os": env::consts::OS,
                "$browser": "smalld_rust",
                "$device": "smalld_rust"
            },
            "intents": self.intents_bitmask,
        });

        if let Err(err) = smalld.send_gateway_payload(Payload::op(Op::Identify).d(d)) {
            warn!("Error sending identify payload: {}", err);
        }
    }

    fn try_resume(&self, smalld: &SmallD) -> Result<(), Error> {
        let sid = self
            .session_id
            .as_ref()
            .ok_or_else(|| Error::illegal_state("No session id to resume with"))?;

        let seq = self
            .sequence_number
            .ok_or_else(|| Error::illegal_state("No sequence number to resume with"))?;

        let d = json!({ "token": self.token, "session_id": sid, "seq": seq });

        if let Err(err) = smalld.send_gateway_payload(Payload::op(Op::Resume).d(d)) {
            warn!("Error sending resume payload: {}", err);
        }

        Ok(())
    }
}
