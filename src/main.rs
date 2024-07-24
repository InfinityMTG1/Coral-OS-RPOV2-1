// src/main.rs
#![no_std] // stop linker from using rust stdlib from linux??
#![no_main] // disable Rust entry points
#![feature(custom_test_frameworks)]
#![test_runner(coral_os::test_runner)]
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

entry_point!(kernel_main);
// boot info struct is used so that the memory map, which is determined during the bootloader
// stage, can be passed to the operating system.
// specifying extern "C" and no_mangle for the entrypoint is no longer necessary because both are
// handled by the entry_point! macro

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use coral_os::memory;
    use x86_64::{structures::paging::Translate, VirtAddr};

    println!("Hello, World!");

    coral_os::init(); // (Currently) intialises the IDT

    // x86_64::instructions::interrupts::int3();

    // use coral_os::memory::active_level_4_table;
    use x86_64::registers::control::Cr3;
    // use x86_64::VirtAddr;

    let (level_4_page_table, _) = Cr3::read();
    println!(
        "Level 4 page table at {:?}",
        level_4_page_table.start_address()
    );

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { memory::init(phys_mem_offset) };

    let addresses = [
        // indentity-mapped vga text mode buffer (physical address == virtual address)
        0xb8000,
        // a code page
        0x201008,
        // a stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
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
    //     use coral_os::print;
    //     print!("-");
    //     for _ in 0..10000 {}
    // }
    coral_os::hlt_loop();
}

// Called upon panic in debug or release configuration
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    coral_os::hlt_loop();
}

// Called upon panic in test configuration
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    coral_os::hlt_loop();
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
