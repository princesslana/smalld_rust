use std::ops::BitOr;

const MAX_SHIFT: u8 = 14;

/// [Gateway intent](https://discord.com/developers/docs/topics/gateway#gateway-intents) to be
/// requested upon identifying with Discord. Configure via
/// [`intents`](crate::smalld::SmallDBuilder#method.intents).
///
/// Intents can be converted to bit masks and combined using `|`.
///
/// ```rust
/// use smalld::Intent;
/// assert_eq!(Intent::GuildMembers as u16, 0b0010);
/// assert_eq!((Intent::GuildMembers | Intent::GuildBans) as u16, 0b0110);
/// assert_eq!((Intent::GuildMembers | Intent::GuildBans | Intent::GuildEmojis) as u16, 0b1110);
/// ```
///
#[derive(Clone, Copy, Debug)]
pub enum Intent {
    Guilds = 1 << 0,
    GuildMembers = 1 << 1,
    GuildBans = 1 << 2,
    GuildEmojis = 1 << 3,
    GuildIntegrations = 1 << 4,
    GuildWebhooks = 1 << 5,
    GuildInvites = 1 << 6,
    GuildVoiceStates = 1 << 7,
    GuildPresences = 1 << 8,
    GuildMessages = 1 << 9,
    GuildMessageReactions = 1 << 10,
    GuildMessageTyping = 1 << 11,
    DirectMessages = 1 << 12,
    DirectMessageReactions = 1 << 13,
    DirectMessageTyping = 1 << 14,
}

impl Intent {
    pub const ALL: u16 = (1 << MAX_SHIFT) - 1;
    pub const PRIVILEGED: u16 = Intent::GuildPresences as u16 | Intent::GuildMembers as u16;
    pub const UNPRIVILEGED: u16 = Intent::ALL ^ Intent::PRIVILEGED;

    pub fn bit_mask_of<I>(intents: I) -> u16
    where
        I: IntoIterator<Item = Intent>,
    {
        intents.into_iter().fold(0, |acc, i| acc | i as u16)
    }
}

impl From<Intent> for u16 {
    fn from(i: Intent) -> u16 {
        i as u16
    }
}

impl BitOr for Intent {
    type Output = u16;

    fn bitor(self, rhs: Intent) -> u16 {
        self as u16 | rhs as u16
    }
}

impl BitOr<Intent> for u16 {
    type Output = u16;

    fn bitor(self, rhs: Intent) -> u16 {
        self | rhs as u16
    }
}
