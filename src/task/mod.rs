pub mod executor;
pub mod keyboard;

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

pub struct Task {
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
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

