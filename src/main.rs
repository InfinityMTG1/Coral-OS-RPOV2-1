// src/main.rs
#![no_std] // stop linker from using rust stdlib from linux??
#![no_main] // disable Rust entry points
#![feature(custom_test_frameworks)]
#![test_runner(ether_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
mod serial;
mod vga_buffer;

pub trait Testable {
    fn run(&self) -> ();
}
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run(); // Use the new implementation of the Testable trait
    }
    exit_qemu(QemuExitCode::Success);
}

//function name will not be mangled
//Program entry point for the linker, which is named start by default
use bootloader::{entry_point, BootInfo};
use ether_os::memory::active_level_4_table;

entry_point!(kernel_main);
// boot info struct is used so that the memory map, which is determined during the bootloader
// stage, can be passed to the operating system.
// specifying extern "C" and no_mangle for the entrypoint is no longer necessary because both are
// handled by the entry_point! macro

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello, World!");

    ether_os::init(); // (Currently) intialises the IDT

    // x86_64::instructions::interrupts::int3();

    // use ether_os::memory::active_level_4_table;
    use x86_64::registers::control::Cr3;
    use x86_64::VirtAddr;

    let (level_4_page_table, _) = Cr3::read();
    println!(
        "Level 4 page table at {:?}",
        level_4_page_table.start_address()
    );

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    for (i, entry) in l4_table.iter().enumerate() {
        use x86_64::structures::paging::PageTable;

        // get the physical address from the entry and convert it
        if !entry.is_unused() {
            let phys = entry.frame().unwrap().start_address();
            let virt = phys.as_u64() + boot_info.physical_memory_offset;
            let ptr = VirtAddr::new(virt).as_mut_ptr();
            let l3_table: &PageTable = unsafe { &*ptr };

            println!("L4 Entry {}: {:?}", i, entry);
            for (i, entry) in l3_table.iter().enumerate() {
                if !entry.is_unused() {
                    println!("L3 Entry {}: {:?}", i, entry);
                }
            }
        }
    }

    // unsafe {
    //     *ptr = 42;
    // }
    // println!("write worked");

    // Only call test_main() when using the test configuration
    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    #[cfg(not(debug_assertions))]
    println!(
        "
|================================================================|
|                 ###\\     Welcome to Coral/OS                   |
|    ####          ###     Written by execat in Rust             |
|    ####           ###    Wishing you a wonderful day ahead!    |
|                    ##)                                         |
|                    ###                                         |
|                    ##)                                         |
|    ####           ###                                          |
|    ####          ###                                           |
|                 ###/                                           |
|================================================================|
    "
    );
    // loop {
    //     use ether_os::print;
    //     print!("-");
    //     for _ in 0..10000 {}
    // }
    ether_os::hlt_loop();
}

// Called upon panic in debug or release configuration
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    ether_os::hlt_loop();
}

// Called upon panic in test configuration
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    ether_os::hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
