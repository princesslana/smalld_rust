use std::thread::sleep;
use std::time::Duration;

pub trait RetryableError {
    fn is_fatal(&self) -> bool;
}

pub fn retry<F, E>(pause: Duration, mut f: F) -> Result<(), E>
where
    F: FnMut() -> Result<(), E>,
    E: RetryableError,
{
    loop {
        match f() {
            Err(err) if err.is_fatal() => break Err(err),
            _ => (),
        }

        sleep(pause);
    }
}
