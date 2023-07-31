use alloc::collections::VecDeque;
use core::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
        RawWaker,
        RawWakerVTable,
        Waker,
    },
};
use alloc::boxed::Box;

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

pub struct Executor {
    task_queue: VecDeque<Task>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            task_queue: VecDeque::new()
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);

            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task is done
                Poll::Pending   => self.task_queue.push_back(task), // task not done, return back into queue
            }
        }
    }
}

fn dummy_raw_waker() -> RawWaker {
    // RawWaker requires to explicitily define virtual method table
    //   that specifies functions that must be called when instance is cloned, woken or dropped
    // Each function receives a *const () argument (type-erased pointer to a value(s)) - should be
    //   this way because RawWaker should be non-generic, but support arbitrary types
    // Typically created for heap-allocated struct that is wrapped into Box or Arc

    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    // (clone, wake, wake_by_ref, drop)
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);

    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    // The behavior of the returned Waker is undefined if the contract defined in RawWaker’s and RawWakerVTable’s documentation is not upheld
    //  RawWakerVTable: 1. These functions must all be thread-safe
    //                  2. Calling one of the contained functions using any other data pointer will cause undefined behavior
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
