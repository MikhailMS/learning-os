use alloc::{
    collections::BTreeMap,
    sync::Arc,
    task::Wake,
};
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
use crossbeam_queue::ArrayQueue;

use crate::task::{
    Task,
    TaskId,
};

pub struct Executor {
    // We use BTree to store Tasks as it allows for fast search
    tasks:       BTreeMap<TaskId, Task>,
    // We use Arc<ArrayQueue> because this queue would be shared between executor and wakers
    //   wakers: push TaskId, executor consumes that TaskId and runs the Task which is associated
    //     with this ID
    task_queue:  Arc<ArrayQueue<TaskId>>,
    // We use BTree to store Wakers as it allows for fast search:
    //   we need to cache Wakers so they could be re-used to wake task multiple times
    //   also ensures that reference-counted wakers are not deallocated inside interrupt handlers
    //     as it may lead to deadlock
    waker_cache: BTreeMap<TaskId, Waker>
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            tasks:       BTreeMap::new(),
            task_queue:  Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;

        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already exists");
        }
        self.task_queue.push(task_id).expect("queue is full");
    }

    pub fn run(&mut self) -> ! {
        // Executor is not optimal as it would run endlesly thus burning CPU at 100%
        loop {
            self.run_ready_tasks();
        }
    }

    fn run_ready_tasks(&mut self) {
        // May not be required as RFC 2229 has been implemented (or so it seems atm)
        let Self {
            tasks,
            task_queue,
            waker_cache
        } = self;

        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None       => continue,
            };

            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));

            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // Task has been completed - remove it together with cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending   => {}
            }
        }
    }
}

struct TaskWaker {
    task_id:    TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>
}

impl TaskWaker {
    // Waker implements From, so we can create safe version of Waker
    // from our Arc-based TaskWaker
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker{
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue is full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

