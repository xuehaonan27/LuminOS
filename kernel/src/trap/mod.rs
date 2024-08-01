use core::arch::global_asm;

use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie, stval,
    stvec::{self, TrapMode},
};

#[cfg(feature = "multiprogramming")]
use crate::task::exit_current_and_run_next;
use crate::{syscall::syscall, task::suspend_current_and_run_next, timer::set_next_trigger};

#[cfg(feature = "batch")]
use crate::batch::run_next_app;

mod context;
pub use context::TrapContext;

#[cfg(feature = "D_EXTENSION_ENABLED")]
global_asm!(include_str!("trap_d_ext.S"));
#[cfg(not(feature = "D_EXTENSION_ENABLED"))]
global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    #[cfg(feature = "profiling")]
    crate::task::user_time_end();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            kprintln!("[kernel] PageFault in application");

            #[cfg(feature = "batch")]
            run_next_app();
            #[cfg(feature = "multiprogramming")]
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            kprintln!("[kernel] IllegalInstruction in application");

            #[cfg(feature = "batch")]
            run_next_app();
            #[cfg(feature = "multiprogramming")]
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            // Schedule next task to run
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}",
                scause.cause(),
                stval
            );
        }
    }
    #[cfg(feature = "profiling")]
    crate::task::user_time_start();
    cx
}

/// Enable timer interrupt.
/// Should be called during kernel initialization. 
pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer() }
}
