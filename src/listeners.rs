use crate::payload::Payload;
use crate::smalld::SmallD;

pub type Listener = dyn Fn(&SmallD, &Payload) + Send + Sync + 'static;

pub struct Listeners {
    listeners: Vec<Box<Listener>>,
}

impl Listeners {
    pub fn new() -> Listeners {
        Listeners {
            listeners: Vec::new(),
        }
    }

    pub fn add<F>(&mut self, f: F)
    where
        F: Fn(&SmallD, &Payload) + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(f));
    }

    pub fn notify(&self, smalld: &SmallD, payload: &Payload) {
        for l in self.listeners.iter() {
            l(smalld, payload);
        }
    }
}
