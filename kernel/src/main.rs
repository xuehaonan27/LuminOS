//! The main module and entrypoint

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use core::arch::global_asm;

#[cfg(feature = "vmm")]
extern crate alloc;
#[cfg(feature = "vmm")]
#[macro_use]
extern crate bitflags;

mod backtracer;
#[cfg(feature = "batch")]
mod batch;
#[cfg(any(
    feature = "multiprogramming",
    feature = "multitasking",
    feature = "vmm"
))]
mod config;
#[macro_use]
mod console;
#[macro_use]
mod debug;
#[cfg(any(
    feature = "multiprogramming",
    feature = "multitasking",
    feature = "vmm"
))]
mod loader;
mod logging;
#[cfg(feature = "vmm")]
mod mm;
mod panic;
mod sbi;
mod sync;
mod syscall;
#[cfg(any(
    feature = "multiprogramming",
    feature = "multitasking",
    feature = "vmm"
))]
mod task;
#[cfg(any(feature = "multitasking", feature = "vmm"))]
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
#[cfg(feature = "batch")]
#[no_mangle]
pub fn rust_main() -> ! {
    logging::init();
    clear_bss();
    logging::init();
    trap::init();
    kprintln!("[kernel] Using batch kernel");
    batch::init();
    batch::run_next_app();
}

/// rust entry-point
#[cfg(feature = "multiprogramming")]
#[no_mangle]
pub fn rust_main() -> ! {
    logging::init();
    clear_bss();
    logging::init();
    trap::init();
    kprintln!("[kernel] Using multiprogramming kernel");
    loader::load_apps();
    task::run_first_task();
    panic!("Unreachable in rust_main!");
}

/// rust entry-point
#[cfg(feature = "multitasking")]
#[no_mangle]
pub fn rust_main() -> ! {
    logging::init();
    clear_bss();
    logging::init();
    trap::init();
    loader::load_apps();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
    panic!("Unreacahble in rust_main!");
}

/// rust entry-point
#[cfg(feature = "vmm")]
#[no_mangle]
pub fn rust_main() -> ! {
    kprintln!("[kernel] Using VMM kernel");
    logging::init();
    clear_bss();
    kprintln!("[kernel] Hello, world!");
    mm::init();
    kprintln!("[kernel] back to world!");
    mm::remap_test();
    task::add_initproc();
    kprintln!("[kernel] after initproc!");
    trap::init();
    //trap::enable_interrupt();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    loader::list_apps();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}

/// rust entry-point
#[cfg(not(any(
    feature = "batch",
    feature = "multiprogramming",
    feature = "multitasking",
    feature = "vmm"
)))]
#[no_mangle]
pub fn rust_main() -> ! {
    logging::init();
    clear_bss();
    sbi::shutdown(false)
}
