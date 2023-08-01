use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{
    pin::Pin,
    task::{ Context, Poll }
};
use futures_util::{
    stream::{
        Stream,
        StreamExt,
    },
    task::AtomicWaker,
};
use pc_keyboard::{
    DecodedKey,
    HandleControl,
    Keyboard,
    ScancodeSet1,
    layouts,
};

use crate::print;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub struct ScancodeStream {
    _private: () // This is to ensure that instance coulld only be created via new() function
}

impl ScancodeStream {
    pub fn new() -> ScancodeStream {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("Scancode::new should only be called once");

        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        // Safe to expect since queue would be initialised at the point where we call poll_next
        let queue = SCANCODE_QUEUE.try_get().expect("queue is not initialised");

        // register() is not free, so if we can pop before,
        // its a wise move
        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode)                   => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub async fn print_keypress() {
    let mut scancode_stream = ScancodeStream::new();
    let mut keyboard        = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    while let Some(scancode) = scancode_stream.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}",   character),
                    DecodedKey::RawKey(key)        => print!("{:?}", key),
                }
            }
        }
    }
}

/// Called by Keyboard interrupt
///
/// Must not block or allocate (thus try_get())
/// pub(crate) is to ensure visibility is only for lib, not main
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Ok(()) = queue.push(scancode) {
            WAKER.wake();
        }
    }
}
