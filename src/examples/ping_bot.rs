use log::debug;
use smalld_rust::SmallD;

fn main() {
    pretty_env_logger::init();

    let mut smalld: SmallD = SmallD::new().expect("Failed to initialize smalld");

    smalld.on_event("MESSAGE_CREATE", |json| {
        if let Some("++ping") = json.get("content").and_then(|c| c.as_str()) {
            // pong
            debug!("Pong!");
        }
    });

    smalld.run();
}
