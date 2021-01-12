use log::info;
use smalld_rust::SmallD;

fn main() {
    pretty_env_logger::init();

    let mut smalld: SmallD = SmallD::new().expect("Failed to initialize smalld");

    smalld.on_gateway_payload(|p| info!("payload received in ping bot! {}", p));

    smalld.run();
}
