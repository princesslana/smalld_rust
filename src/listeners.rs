pub type Listener<T> = dyn Fn(&T) + Send + Sync + 'static;

pub struct Listeners<T> {
    listeners: Vec<Box<Listener<T>>>,
}

impl<T> Listeners<T> {
    pub fn new() -> Listeners<T> {
        Listeners {
            listeners: Vec::new(),
        }
    }

    pub fn add<F>(&mut self, f: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(f));
    }

    pub fn notify(&self, t: T) {
        for l in self.listeners.iter() {
            l(&t)
        }
    }
}
