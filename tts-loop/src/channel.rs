use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub enum Item<T> {
    Some(T),
    Cancel,
}

struct Inner<T> {
    queue: Mutex<VecDeque<Item<T>>>,
    cond: Condvar,
}

pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Sender<T> {
    pub fn send(&self, val: T) {
        let mut queue = self.inner.queue.lock().expect("Poisoned lock");
        queue.push_back(Item::Some(val));
        self.inner.cond.notify_one();
    }

    pub fn cancel(&self) {
        let mut queue = self.inner.queue.lock().expect("Poisoned lock");
        queue.clear();
        queue.push_back(Item::Cancel);
        self.inner.cond.notify_one();
    }
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Item<T> {
        let mut queue = self.inner.queue.lock().expect("Poisoned lock");
        loop {
            if let Some(val) = queue.pop_front() {
                return val;
            }

            queue = self.inner.cond.wait(queue).expect("Poisoned lock");
        }
    }

    pub fn peek_cancel(&self) -> bool {
        let queue = self.inner.queue.lock().expect("Poisoned lock");
        if let Some(Item::Cancel) = queue.front() {
            return true;
        }

        false
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let queue = Mutex::new(VecDeque::new());
    let cond = Condvar::new();

    let inner = Inner { queue, cond };
    let inner = Arc::new(inner);

    let tx = Sender {
        inner: Arc::clone(&inner),
    };

    let rx = Receiver {
        inner: Arc::clone(&inner),
    };

    (tx, rx)
}
