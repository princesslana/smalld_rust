use log::{debug, warn};
use serde_json::json;
use smalld_rust::SmallD;

fn main() {
    pretty_env_logger::init();

    let smalld = SmallD::new().expect("Failed to initialize smalld");

    smalld.on_event("MESSAGE_CREATE", move |json| {
        if let Some("++ping") = json.get("content").and_then(|c| c.as_str()) {
            debug!("Pong!");
            if let Some(channel_id) = json.get("channel_id").and_then(|c| c.as_str()) {
                if let Err(err) = smalld.post(
                    format!("/channels/{}/messages", channel_id),
                    json!({"content":"pong"}),
                ) {
                    warn!("Error sending pong response: {}", err);
                }
            }
        }
    });

    smalld.run();
}
