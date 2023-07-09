use x86_64::structures::idt::{ InterruptDescriptorTable, InterruptStackFrame };
use lazy_static::lazy_static;
use crate::println;


lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_hanlder);

        idt
    };
}

pub fn init_dft() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_hanlder(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}


#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
