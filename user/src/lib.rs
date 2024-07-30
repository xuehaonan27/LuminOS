#![no_std]
#![feature(linkage)] // manage linker behavior
#![feature(panic_info_message)]

use syscall::{sys_exit, sys_write};

mod syscall;
#[macro_use]
pub mod console;
mod panic;

#[no_mangle]
#[link_section = ".text.entry"] // put `_start` in `.text.entry` section
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    panic!("unreachable after sys_exit!");
}

/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }
    (start_bss as usize..end_bss as usize).for_each(|x| unsafe {
        (x as *mut u8).write_volatile(0);
    })
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
