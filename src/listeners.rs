use crate::payload::Payload;
use crate::smalld::SmallD;

pub type Listener = dyn FnMut(&SmallD, &Payload) + Send + Sync + 'static;

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
        F: FnMut(&SmallD, &Payload) + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(f));
    }

    pub fn notify(&mut self, smalld: &SmallD, payload: &Payload) {
        for l in self.listeners.iter_mut() {
            l(smalld, payload);
        }
    }
}
