use smalld_rust::SmallD;

fn main() {
    pretty_env_logger::init();

    let smalld: SmallD = SmallD::new().expect("Failed to initialize smalld");

    smalld.run();
}
