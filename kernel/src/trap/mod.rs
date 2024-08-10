use core::arch::{asm, global_asm};

use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie, stval,
    stvec::{self, TrapMode},
};

use crate::task::{
    current_trap_cx, current_user_token, exit_current_and_run_next, suspend_current_and_run_next,
};
use crate::timer::set_next_trigger;
use crate::{
    config::{TRAMPOLINE, TRAP_CONTEXT},
    task::{current_task, SignalFlags},
};
use crate::{
    syscall::syscall,
    task::{check_signals_error_of_current, handle_signals},
};

mod context;
pub use context::TrapContext;

#[cfg(feature = "D_EXTENSION_ENABLED")]
global_asm!(include_str!("trap_d_ext.S"));
#[cfg(not(feature = "D_EXTENSION_ENABLED"))]
global_asm!(include_str!("trap.S"));

pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_from_kernel() -> ! {
    use riscv::register::sepc;
    kprintln!("stval = {:#x}, sepc = {:#x}", stval::read(), sepc::read());
    panic!("a trap {:?} from kernel!", scause::read().cause());
}

#[no_mangle]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}

#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            let mut cx = current_trap_cx();
            cx.sepc += 4;
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
            // cx is changed during sys_exec, so we have to call it again
            cx = current_trap_cx();
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            kprintln!("[kernel] PageFault in application");
            // exit_current_and_run_next(-2);
            current_add_signal(SignalFlags::SIGSEGV);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            kprintln!("[kernel] IllegalInstruction in application");
            // exit_current_and_run_next(-3);
            current_add_signal(SignalFlags::SIGSEGV);
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

    // handle signals (handle the sent signal)
    handle_signals();

    // check error signals (if error then exit)
    if let Some((errno, msg)) = check_signals_error_of_current() {
        kprintln!("[kernel] {}", msg);
        exit_current_and_run_next(errno);
    }

    trap_return();
}

/// Enable timer interrupt.
/// Should be called during kernel initialization.
pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer() }
}

pub fn current_add_signal(signal: SignalFlags) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.signals |= signal;
}
