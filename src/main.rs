// src/main.rs
#![no_std] // stop linker from using rust stdlib from linux??
#![no_main] // disable Rust entry points

use core::panic::PanicInfo;

mod vga_buffer;

// Called upon panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
#[no_mangle] //function name will not be mangled
             //Program entry point for the linker, which is named start by default
pub extern "C" fn _start() -> ! {
    println!("Hello, World!");
    panic!("Some panic message");
    loop {}
}
