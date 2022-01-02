use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
pub enum Request {
    SetText { text: String },
    LogStart { num_iters: i32 },
    RunTts,
    PlayAudio,
    RunStt,
    SetVoice { voice: String },
    EnableAudio { enable: bool },
    Cancel,
    Shutdown,
}

impl Request {
    fn is_priority_action(&self) -> bool {
        // Priority actions will always happen before non-priority actions. Priority actions cannot be canceled
        match *self {
            Request::SetText { .. }
            | Request::LogStart { .. }
            | Request::RunTts
            | Request::RunStt
            | Request::PlayAudio => false,
            Request::SetVoice { .. }
            | Request::Cancel
            | Request::EnableAudio { .. }
            | Request::Shutdown => true,
        }
    }
}

#[derive(Debug)]
struct Queues {
    priority: VecDeque<Request>,
    regular: VecDeque<Request>,
}

struct Inner {
    queues: Mutex<Queues>,
    cond: Condvar,
}

pub struct Sender {
    inner: Arc<Inner>,
}

impl Sender {
    pub fn send(&self, val: Request) {
        let mut queues = self.inner.queues.lock().expect("Poisoned lock");
        if let Request::Cancel = val {
            // Push cancel to both queues. In priority it's used to indicate
            // that we need to execute a cancel, in the regular queue it tells
            // us when to stop
            queues.regular.push_back(Request::Cancel);
            queues.priority.push_back(Request::Cancel);
        } else if val.is_priority_action() {
            queues.priority.push_back(val);
        } else {
            queues.regular.push_back(val);
        }
        self.inner.cond.notify_one();
    }
}

pub struct Receiver {
    inner: Arc<Inner>,
}

impl Receiver {
    pub fn recv(&self) -> Request {
        let mut queues = self.inner.queues.lock().expect("Poisoned lock");
        loop {
            if let Some(val) = queues.priority.pop_front() {
                return val;
            } else if let Some(val) = queues.regular.pop_front() {
                return val;
            }

            queues = self.inner.cond.wait(queues).expect("Poisoned lock");
        }
    }

    pub fn execute_cancel(&self) -> bool {
        let mut queues = self.inner.queues.lock().expect("Poisoned lock");

        let mut any_removed = false;

        while let Some(val) = queues.regular.pop_front() {
            if let Request::Cancel = val {
                break;
            } else {
                any_removed = true
            }
        }

        any_removed
    }
}

pub fn channel() -> (Sender, Receiver) {
    let queues = Queues {
        priority: VecDeque::new(),
        regular: VecDeque::new(),
    };
    let queues = Mutex::new(queues);

    let cond = Condvar::new();

    let inner = Inner { queues, cond };
    let inner = Arc::new(inner);

    let tx = Sender {
        inner: Arc::clone(&inner),
    };

    let rx = Receiver {
        inner: Arc::clone(&inner),
    };

    (tx, rx)
}
