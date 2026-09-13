use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Starting,
    Ready,
    Draining,
    Stopped,
}

pub struct Controller {
    requested: Arc<AtomicBool>,
    pub deadline: Duration,
}

impl Controller {
    pub fn new(deadline: Duration) -> Self {
        Self {
            requested: Arc::new(AtomicBool::new(false)),
            deadline,
        }
    }
    pub fn signal(&self) {
        self.requested.store(true, Ordering::Release);
    }
    pub fn is_requested(&self) -> bool {
        self.requested.load(Ordering::Acquire)
    }
    pub fn flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.requested)
    }
    pub fn deadline_from(&self, start: Instant) -> Instant {
        start + self.deadline
    }
}

pub fn watch_file(path: PathBuf, requested: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !requested.load(Ordering::Acquire) {
            if path.exists() {
                requested.store(true, Ordering::Release);
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shutdown_is_idempotent_and_deadline_is_deterministic() {
        let controller = Controller::new(Duration::from_millis(500));
        let start = Instant::now();
        controller.signal();
        controller.signal();
        assert!(controller.is_requested());
        assert!(controller.deadline_from(start) >= start + Duration::from_millis(500));
    }
}
