use crate::smalld::Event;

pub type Listener = dyn Fn(&Event<'_>) + Send + Sync + 'static;

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
        F: Fn(&Event<'_>) + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(f));
    }

    pub fn notify(&self, t: &Event) {
        for l in self.listeners.iter() {
            l(t)
        }
    }
}
