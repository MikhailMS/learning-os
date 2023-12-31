use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::{
    instructions::port::Port,
    structures::idt::{ InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode },
    registers::control::Cr2
};

use crate::{
    task::keyboard::add_scancode,
    gdt,
    hlt_loop,
    println,
};

pub const PIC1_OFFSET: u8 = 32;
pub const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET)
});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC1_OFFSET,
    Keyboard,
    SecondaryIC,
    SerialPort2,
    SerialPort1,
    ParallelPort23,
    FloppyDisk,
    ParallelPort1,
    RealTimeClock,
    ACPI,
    Available1,
    Available2,
    Mouse,
    CoProcessor,
    PrimaryAta,
    SecondaryAta
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Set CPU Interrupts
        idt.breakpoint.set_handler_fn(breakpoint_hanlder);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_IDX);
        }

        idt.page_fault.set_handler_fn(page_fault_handler);

        // Set PIC interrupts
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_hanlder(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error code: {:?}",       error_code);
    println!("{:#?}",                  stack_frame);

    hlt_loop();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // print!(".");

    // Unsafe because notify_end_of_interrupt() can potentially cancel unsent interrupt or cause system to hang
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut ps2_port = Port::new(0x60); // 0x0060-0x0064: The "8042" PS/2 Controller or its predecessors, dealing with keyboards and mice
    let scancode: u8 = unsafe { ps2_port.read() };

    // Send scancode into background async task - keeping interrupt handler as simple as possible
    add_scancode(scancode);

    // Unsafe because notify_end_of_interrupt() can potentially cancel unsent interrupt or cause system to hang
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}


#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
