const MAX_SHIFT: u8 = 14;

#[derive(Clone, Copy)]
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

    pub fn to_bit_mask<I>(intents: I) -> u16
    where
        I: IntoIterator<Item = Intent>,
    {
        intents.into_iter().fold(0, |acc, i| acc | i as u16)
    }
}
