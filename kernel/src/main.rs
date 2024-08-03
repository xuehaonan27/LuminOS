//! The main module and entrypoint

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::global_asm;
use log::*;

mod backtracer;
#[cfg(feature = "batch")]
mod batch;
#[cfg(any(feature = "multiprogramming", feature = "multitasking"))]
mod config;
#[macro_use]
mod console;
#[macro_use]
mod debug;
#[cfg(any(feature = "multiprogramming", feature = "multitasking"))]
mod loader;
mod logging;
mod panic;
mod sbi;
mod sync;
mod syscall;
#[cfg(any(feature = "multiprogramming", feature = "multitasking"))]
mod task;
#[cfg(feature = "multitasking")]
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
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn boot_stack_lower_bound();
        fn boot_stack_top();
    }
    clear_bss();
    logging::init();
    kprintln!("[kernel] Hello, world!");

    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as usize,
        etext as usize
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    warn!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower_bound as usize
    );
    error!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);

    trap::init();

    #[cfg(feature = "batch")]
    {
        kprintln!("[kernel] Using batch kernel");
        batch::init();
        batch::run_next_app();
    }
    #[cfg(feature = "multiprogramming")]
    {
        kprintln!("[kernel] Using multiprogramming kernel");
        loader::load_apps();
        task::run_first_task();
        panic!("Unreachable in rust_main!");
    }
    #[cfg(feature = "multitasking")]
    {
        loader::load_apps();
        trap::enable_timer_interrupt();
        timer::set_next_trigger();

        {
            use riscv::register::sstatus;
            unsafe { sstatus::set_sie() }; // enable kernel mode interrupt
            loop { // sanity check
                if trap::check_kernel_interrupt() {
                    kprintln!("kernel interrupt returned.");
                    break;
                }
            }
            unsafe {sstatus::clear_sie()}; // disable kernel mode interrupt
        }

        task::run_first_task();
        panic!("Unreacahble in rust_main!");
    }
    #[cfg(not(any(
        feature = "batch",
        feature = "multiprogramming",
        feature = "multitasking"
    )))]
    sbi::shutdown(false)
}
