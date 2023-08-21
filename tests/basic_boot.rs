#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ether_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use ether_os::println;

#[no_mangle] // don't mangle the function name during compilation
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ether_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
