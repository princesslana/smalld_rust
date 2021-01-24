use log::warn;
use serde_json::json;
use smalld::SmallD;

fn main() {
    pretty_env_logger::init();

    let smalld = SmallD::new().expect("Failed to initialize smalld");

    smalld.on_event("MESSAGE_CREATE", move |smalld, json| {
        if let Some("++ping") = json["content"].as_str() {
            if let Some(channel_id) = json["channel_id"].as_str() {
                if let Err(err) = smalld
                    .resource(format!("/channels/{}/messages", channel_id))
                    .post(json!({"content":"pong"}))
                {
                    warn!("Error sending pong response: {}", err);
                }
            }
        }
    });

    smalld.run();
}
