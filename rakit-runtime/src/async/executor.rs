use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use crate::hook::RakitValue;

thread_local! {
    pub static EXECUTOR: RefCell<RakitExecutor> = RefCell::new(RakitExecutor::new());
}

pub struct RakitExecutor {
    ready_queue: VecDeque<Pin<Box<dyn Future<Output = RakitValue>>>>,
    waiting: Vec<(u64, Pin<Box<dyn Future<Output = RakitValue>>>)>,
}

impl RakitExecutor {
    pub fn new() -> Self {
        RakitExecutor {
            ready_queue: VecDeque::new(),
            waiting: Vec::new(),
        }
    }

    pub fn spawn(&mut self, future: Pin<Box<dyn Future<Output = RakitValue>>>) {
        self.ready_queue.push_back(future);
    }

    pub fn spawn_for_fiber(
        &mut self,
        fiber_id: u64,
        future: Pin<Box<dyn Future<Output = RakitValue>>>,
    ) {
        self.waiting.push((fiber_id, future));
    }

    pub fn tick(&mut self) -> usize {
        let mut completed = 0;
        let mut i = 0;
        while i < self.ready_queue.len() {
            let mut future = self.ready_queue.remove(i).unwrap();
            let waker = noop_waker();
            let mut ctx = Context::from_waker(&waker);
            match future.as_mut().poll(&mut ctx) {
                Poll::Ready(_) => {
                    completed += 1;
                }
                Poll::Pending => {
                    self.ready_queue.push_back(future);
                    i += 1;
                }
            }
        }
        completed
    }

    pub fn run_until_complete(&mut self) {
        while !self.ready_queue.is_empty() || !self.waiting.is_empty() {
            self.tick();
            let mut still_waiting = Vec::new();
            for (fiber_id, mut future) in self.waiting.drain(..) {
                let waker = noop_waker();
                let mut ctx = Context::from_waker(&waker);
                match future.as_mut().poll(&mut ctx) {
                    Poll::Ready(value) => {
                        crate::r#async::suspense::resolve_suspense(fiber_id, value);
                    }
                    Poll::Pending => {
                        still_waiting.push((fiber_id, future));
                    }
                }
            }
            self.waiting = still_waiting;
        }
    }
}

fn noop_waker() -> Waker {
    let raw_waker = raw_noop_waker();
    unsafe { Waker::from_raw(raw_waker) }
}

fn raw_noop_waker() -> RawWaker {
    fn noop_clone(_: *const ()) -> RawWaker {
        raw_noop_waker()
    }
    fn noop(_: *const ()) {}
    RawWaker::new(
        std::ptr::null(),
        &RawWakerVTable::new(noop_clone, noop, noop, noop),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_completes_future() {
        let mut executor = RakitExecutor::new();
        let future = Box::pin(async { RakitValue::Text("selesai".to_string()) });
        executor.spawn(future);
        executor.run_until_complete();
        assert!(executor.ready_queue.is_empty());
    }

    #[test]
    fn test_executor_tick_returns_count() {
        let mut executor = RakitExecutor::new();
        let future = Box::pin(async { RakitValue::Null });
        executor.spawn(future);
        let count = executor.tick();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_executor_multiple_futures() {
        let mut executor = RakitExecutor::new();
        for _ in 0..5 {
            let future = Box::pin(async { RakitValue::Number(42.0) });
            executor.spawn(future);
        }
        executor.run_until_complete();
        assert!(executor.ready_queue.is_empty());
    }
}
