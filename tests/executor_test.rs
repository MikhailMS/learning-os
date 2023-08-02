#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

use alloc::string::String;
use bootloader::{ entry_point, BootInfo };
use core::fmt;
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
    qemu_codes,
    serial_println,
};
use x86_64::VirtAddr;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let expected_output = "async number: 42";
    let mut output      = String::new();

    if let Some(msg) = info.message() {
        fmt::write(&mut output, *msg);
    }

    if expected_output == output {
        serial_println!("executor_test::simple_task_completes... \n[ok]!");
        qemu_codes::exit_qemu(qemu_codes::QemuExitCode::Success);
    }
    qemu_codes::exit_qemu(qemu_codes::QemuExitCode::Failure);

    loop {}
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

    simple_task_completes();
    serial_println!("[test did not panic]");
    qemu_codes::exit_qemu(qemu_codes::QemuExitCode::Failure);

    loop {}
}

fn simple_task_completes() {
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(read_task()));

    executor.run();
}


async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

async fn read_task() {
    let assumed_output    = "async number: 42";
    let writer            = WRITER.lock();
    let mut actual_output = String::new();

    // TODO: re-write to make it better :)
    for (i, _c) in assumed_output.chars().enumerate() {
        let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
        fmt::write(&mut actual_output, format_args!("{}", char::from(screen_char.ascii_char)));
    }

    panic!("{}", actual_output);
}

