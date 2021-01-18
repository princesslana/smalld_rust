# smalld_rust

[![Crates.io](https://img.shields.io/crates/v/smalld_rust)](https://crates.io/crates/smalld_rust)
[![docs.rs](https://docs.rs/smalld_rust/badge.svg)](https://docs.rs/smalld_rust)
![Build](https://github.com/princesslana/smalld_rust/workflows/Build/badge.svg)
[![Discord](https://img.shields.io/discord/417389758470422538)](https://discord.gg/3aTVQtz)

SmallD aims to be a minmalist client for the Discord API. It aims to let you use the Discord API, without hiding or abstracting it.

## Installing

smalld_rust is published on [crates.io](https://crates.io/crates/smalld_rust).
Add to the dependencies section of your Cargo.toml.

```toml
[dependencies]
smalld_rust = "*"
```

To use the latest development version add as a git dependency.

```toml
[dependencies]
smalld_rust = { git = "https://github.com/princesslana/smalld_rust", branch = "main" }
```

## Documentation

Documentation is published to [docs.rs](https://docs.rs/smalld_rust).
Help is also available on the [Discord Projects Hub](https://discord.gg/3aTVQtz) Discord server.

## Examples

To run the example ping bot:

```console
$ SMALLD_TOKEN=<discord bot token> RUST_LOG=info cargo run --example ping_bot
```

**WARNING:** Enabling debug level logging will output your token in the log messages, so be careful where you send those.

## Contact and Contributing

Reach out to the [Discord Projects Hub](https://discord.gg/3aTVQtz) on Discord and look for the smalld_rust channel.
