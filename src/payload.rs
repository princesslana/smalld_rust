use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

///  Struct representing [payloads](https://discord.com/developers/docs/topics/gateway#payloads)
///  received from the Discord gateway.
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
            op,
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

/// [Opcode](https://discord.com/developers/docs/topics/opcodes-and-status-codes#gateway-opcodes)
/// as will be received in a [`Payload`](Payload) received from Discord.
#[derive(Clone, Copy, Debug)]
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
        use Op::*;
        match op {
            0 => Dispatch,
            1 => Heartbeat,
            2 => Identify,
            3 => PresenceUpdate,
            4 => VoiceStateUpdate,
            6 => Resume,
            7 => Reconnect,
            8 => RequestGuildMembers,
            9 => InvalidSession,
            10 => Hello,
            11 => HeartbeatAck,
            n => Unknown(n),
        }
    }
}

impl From<Op> for u8 {
    fn from(op: Op) -> Self {
        use Op::*;
        match op {
            Dispatch => 0,
            Heartbeat => 1,
            Identify => 2,
            PresenceUpdate => 3,
            VoiceStateUpdate => 4,
            Resume => 6,
            Reconnect => 7,
            RequestGuildMembers => 8,
            InvalidSession => 9,
            Hello => 10,
            HeartbeatAck => 11,
            Unknown(n) => n,
        }
    }
}

impl Serialize for Op {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8((*self).into())
    }
}

impl<'a> Deserialize<'a> for Op {
    fn deserialize<D>(deserializer: D) -> Result<Op, D::Error>
    where
        D: Deserializer<'a>,
    {
        Ok(u8::deserialize(deserializer)?.into())
    }
}
