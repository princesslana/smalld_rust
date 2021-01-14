use serde::de::Deserializer;
use serde::de::{self, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Deserialize, Serialize, Debug)]
pub struct Payload {
    pub op: Op,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<u64>,
}

impl Payload {
    pub fn op(op: Op) -> Payload {
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

#[derive(Debug)]
pub enum Op {
    Dispatch,
    Heartbeat,
    Identify,
    PresenceUpdate,
    VoiceStateUpdate,
    Resume,
    Reconnect,
    RequestGuildMembers,
    InvalidSession,
    Hello,
    HeartbeatAck,
    Unknown(u8),
}

impl From<u8> for Op {
    fn from(op: u8) -> Self {
        match op {
            0 => Op::Dispatch,
            1 => Op::Heartbeat,
            2 => Op::Identify,
            3 => Op::PresenceUpdate,
            4 => Op::VoiceStateUpdate,
            6 => Op::Resume,
            7 => Op::Reconnect,
            8 => Op::RequestGuildMembers,
            9 => Op::InvalidSession,
            10 => Op::Hello,
            11 => Op::HeartbeatAck,
            n => Op::Unknown(n),
        }
    }
}

impl From<&Op> for u8 {
    fn from(op: &Op) -> Self {
        match op {
            Op::Dispatch => 0,
            Op::Heartbeat => 1,
            Op::Identify => 2,
            Op::PresenceUpdate => 3,
            Op::VoiceStateUpdate => 4,
            Op::Resume => 6,
            Op::Reconnect => 7,
            Op::RequestGuildMembers => 8,
            Op::InvalidSession => 9,
            Op::Hello => 10,
            Op::HeartbeatAck => 11,
            Op::Unknown(n) => *n,
        }
    }
}

impl Serialize for Op {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.into())
    }
}
struct OpVisitor;

impl<'a> Visitor<'a> for OpVisitor {
    type Value = Op;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an unsigned integer")
    }

    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.into())
    }
}

impl<'a> Deserialize<'a> for Op {
    fn deserialize<D>(deserializer: D) -> Result<Op, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_u8(OpVisitor)
    }
}
