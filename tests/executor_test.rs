#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(radius_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{ entry_point, BootInfo };
use core::panic::PanicInfo;
use radius_os::{
    memory::{ self, BootInfoFrameAllocator },
    task::{
        executor::Executor,
        Task,
    },
    vga::{ WRITER, BUFFER_HEIGHT },
    allocator,
    init,
    println,
    test_panic_handler,
};
use x86_64::VirtAddr;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {

    init();

    // Initialise Heap space >>>>>>>
    let phys_memory_offset  = VirtAddr::new(boot_info.physical_memory_offset);
    // Initialise mapper (virt to phys)
    let mut mapper          = unsafe { memory::init(phys_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialisation failed");
    // <<<<<<<<

    test_main();
    loop {}
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}


#[test_case]
fn simple_task_completes() {
    use x86_64::instructions::interrupts;
    let expected_output = "async number: 42";
    
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));

    executor.run();

    interrupts::without_interrupts(|| {
        let writer = WRITER.lock();

        for (i, c) in expected_output.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_char), c);
        }
    });
}

