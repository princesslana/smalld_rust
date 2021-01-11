use log::error;
use smalld_rust::SmallD;
use std::process::exit;

fn main() {
    pretty_env_logger::init();

    match SmallD::new() {
        Ok(smalld) => smalld.run(),
        Err(e) => {
            error!("Application error: {}", e);
            exit(1);
        }
    }
}
