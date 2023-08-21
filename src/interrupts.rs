use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

// Set up the IDT for reference anywhere in the file so it has
// 'static lifetime and can be manipulated.
// Behind the scenes, an unsafe block is used, but it is wrapped
// safely in the lazy_static macro
// Sets the breakpoint exception handler function to 'breakpoint_handler'
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

// Loads the IDT set up above
pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    x86_64::instructions::interrupts::int3();
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    // Invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}
