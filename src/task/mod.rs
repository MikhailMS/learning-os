pub mod executor;
pub mod keyboard;

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{
        AtomicU64,
        Ordering,
    },
    task::{
        Context,
        Poll,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        // We only require Id to be unique, so having Relaxed ordering is enough (weakest ordering available)
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task {
    id: TaskId,
    // () - means that Task doesn't return and we only require tasks side effect
    // dyn Future - means that we are going to store trait object
    // Pin<Box> - means that value is on heap and cannot be moved in memory; also &mut reference
    // cannot be created
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    // 'static is required because we need to ensure that future lives as long as the Task
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

