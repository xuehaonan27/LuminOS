#![no_std]
#![feature(linkage)] // manage linker behavior
#![feature(panic_info_message)]

use syscall::*;

mod syscall;
#[macro_use]
pub mod console;
mod panic;

#[no_mangle]
#[link_section = ".text.entry"] // put `_start` in `.text.entry` section
pub extern "C" fn _start() -> ! {
    exit(main());
    panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}

pub fn yield_() -> isize {
    sys_yield()
}

pub fn get_time() -> isize {
    sys_get_time()
}
