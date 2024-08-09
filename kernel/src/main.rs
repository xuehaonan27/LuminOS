//! The main module and entrypoint

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use core::arch::global_asm;

extern crate alloc;

#[macro_use]
extern crate bitflags;

mod backtracer;
#[path = "boards/qemu.rs"]
mod board;
mod config;
#[macro_use]
mod console;
#[macro_use]
mod debug;
mod drivers;
mod fs;
mod loader;
mod logging;
mod mm;
mod panic;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|x| unsafe {
        (x as *mut u8).write_volatile(0);
    })
}

/// rust entry-point
#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    kprintln!("[kernel] Hello, world!");
    mm::init();
    mm::remap_test();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    fs::list_apps();
    task::add_initproc();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
