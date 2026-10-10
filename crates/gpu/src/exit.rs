//! Process shutdown gate for GPU work.
//!
//! GPU devices are process statics, while preview workers may run on other threads. Closing the
//! gate before process teardown prevents late driver entry, then waits for work already inside the
//! driver up to a caller supplied deadline.

use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

struct State {
    closed: bool,
    busy: usize,
}

pub(crate) struct Gate {
    state: Mutex<State>,
    idle: Condvar,
}

pub(crate) struct Work<'a>(&'a Gate);

static GATE: Gate = Gate::new();

pub(crate) fn enter() -> Option<Work<'static>> {
    GATE.enter()
}

pub(crate) fn begin_shutdown() {
    GATE.close();
}

pub(crate) fn shutting_down() -> bool {
    GATE.lock().closed
}

pub(crate) fn wait_idle(timeout: Duration) -> bool {
    GATE.wait_idle(timeout)
}

impl Gate {
    pub const fn new() -> Self {
        Self { state: Mutex::new(State { closed: false, busy: 0 }), idle: Condvar::new() }
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn enter(&self) -> Option<Work<'_>> {
        let mut state = self.lock();
        if state.closed {
            return None;
        }
        state.busy = state.busy.saturating_add(1);
        Some(Work(self))
    }

    pub fn close(&self) {
        self.lock().closed = true;
    }

    pub fn wait_idle(&self, timeout: Duration) -> bool {
        let started = Instant::now();
        let mut state = self.lock();
        while state.busy > 0 {
            let Some(left) = timeout.checked_sub(started.elapsed()).filter(|duration| !duration.is_zero()) else {
                return false;
            };
            state = self.idle.wait_timeout(state, left).unwrap_or_else(|e| e.into_inner()).0;
        }
        true
    }
}

impl Drop for Work<'_> {
    fn drop(&mut self) {
        let mut state = self.0.lock();
        state.busy = state.busy.saturating_sub(1);
        if state.busy == 0 {
            drop(state);
            self.0.idle.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_blocks_new_work_and_waits_for_existing_work() {
        let gate = Gate::new();
        assert!(gate.wait_idle(Duration::ZERO));
        let outer = gate.enter().expect("gate starts open");
        let inner = gate.enter().expect("nested work is allowed");
        gate.close();
        assert!(gate.enter().is_none());
        drop(inner);
        assert!(!gate.wait_idle(Duration::from_millis(1)));
        std::thread::scope(|scope| {
            scope.spawn(move || {
                std::thread::sleep(Duration::from_millis(10));
                drop(outer);
            });
            assert!(gate.wait_idle(Duration::from_secs(1)));
        });
        assert!(gate.enter().is_none());
    }

    #[test]
    fn work_guard_leaves_gate_after_unwind() {
        let gate = Gate::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _work = gate.enter();
            panic!("expected test unwind");
        }));
        assert!(result.is_err());
        assert!(gate.wait_idle(Duration::ZERO));
    }
}
