//! SmallD aims to be a minmalist client for the Discord API. It aims to let you use the Discord
//! API, without hiding or abstracting it.
//!
//! SmallD takes care of the essentials of interacting with the Discord API, providing a small
//! and flexible core to be built upon. As it does not aim to hide the Discord API working with
//! SmallD will also require an understanding of Discord's API. As such it's usually helpful to
//! keep the [Discord Dev Docs](https://discord.com/developers/docs/intro) in mind.
//!
//! An example of what's in scope for SmallD:
//!   * Authentication (Identifying/Resuming)
//!   * Rate limiting
//!   * Handling disconnections/reconnects
//!
//! Examples of what is not in scope for SmallD:
//!   * Caching
//!   * Command Framework
//!
//! # Getting Started
//!
//! After you have [created a new
//! project](https://doc.rust-lang.org/cargo/guide/creating-a-new-project.html)
//! add smalld_rust as a dependency.
//!
//! ```toml
//! [dependencies]
//! smalld_rust = "*"
//! ```
//!
//! To use SmallD create a SmallD instance and call [`run`](smalld::SmallD#method.run) to connect to
//! Discord.
//!
//! ```no_run
//! use smalld::SmallD;
//!
//! let smalld = SmallD::new().expect("Failed to initialize smalld");
//!
//! // this will block and run until a fatal error or smalld.close() is called
//! smalld.run();
//! ```
//!
//! By default this will look for your Discord token in the `SMALLD_TOKEN` environment variable.
//! It's possible to explicitly provide the token and other configuration options when using
//! [`SmallDBuilder`](smalld::SmallDBuilder).
//!
//! ```no_run
//! use smalld::{Intent, SmallDBuilder};
//!
//! let smalld = SmallDBuilder::new()
//!   .token("my_discord_token")
//!   .intents(Intent::GuildMessages | Intent::DirectMessages)
//!   .build()
//!   .expect("Failed to initialize smalld");
//!
//! smalld.run();
//! ```
//!
//! To listen to events from Discord use the [`on_event`](smalld::SmallD#method.on_event) method,
//! or for all gateway payloads use
//! [`on_gateway_payload`](smalld::SmallD#method.on_gateway_payload), and attach a listener.
//! Each listener receives a reference to [`SmallD`](smalld::SmallD) and the json
//! [`Value`](https://docs.serde.rs/serde_json/value/enum.Value.html) associated with that event.
//!
//! ```no_run
//! use smalld::SmallD;
//!
//! let smalld = SmallD::new().expect("Failed to initialize smalld");
//!
//! smalld.on_event("MESSAGE_CREATE", |smalld, json| {
//!   if let Some("ping") = json["content"].as_str() {
//!     println!("Ping Received!");
//!   }
//! });
//!
//! smalld.run();
//! ```
//!   
//! To send requests through Discord's resources api SmallD provies the
//! [`resource`](smallD::SmallD#method.resource) method. It accepts the path of the resource and
//! provides a builder like interface that allows adding query parameters and calling the resource.
//! We can use this method to send a request to the [create
//! message](https://discord.com/developers/docs/resources/channel#create-message) endpoint.
//!
//! For example, we could add the following method to send a pong response for the above example.
//!
//! ```no_run
//! # use smalld::{SmallD, Error};
//! # use serde_json::{json, Value};
//! pub fn send_pong(smalld: &SmallD, reply_to: Value) -> Result<(), Error> {
//!   if let Some(channel_id) = reply_to["channel_id"].as_str() {
//!     smalld.resource(format!("/channels/{}/msesages", channel_id))
//!           .post(json!({"content" : "pong"}))?;
//!   };
//!
//!   Ok(())
//! }
//! ```

pub use crate::error::Error;
pub use crate::intents::Intent;
pub use crate::payload::{Op, Payload};
pub use crate::resource::Resource;
pub use crate::smalld::{SmallD, SmallDBuilder};

mod error;
mod gateway;
mod heartbeat;
mod http;
mod identify;
mod intents;
mod listeners;
mod payload;
mod resource;
mod retry;
mod smalld;
