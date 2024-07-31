use core::arch::global_asm;

use riscv::register::{
    scause::{self, Exception, Trap},
    sstatus, stval,
    stvec::{self, TrapMode},
};

use crate::syscall::syscall;
#[cfg(feature = "multiprogramming")]
use crate::task::exit_current_and_run_next;

#[cfg(feature = "batch")]
use crate::batch::run_next_app;

#[repr(C)]
pub struct TrapContext {
    /// General purpose registers
    pub x: [usize; 32],

    /// S Mode sstatus register
    pub sstatus: usize,

    /// S Mode sepc register
    pub sepc: usize,
}

#[allow(unused)]
impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    pub fn init_context(entry: usize, sp: usize) -> Self {
        let sstatus = sstatus::read();
        let mut sstatus: usize = unsafe { core::mem::transmute(sstatus) };
        // set SPP to user mode
        sstatus &= !(1 << 8);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
        };
        cx.set_sp(sp);
        cx
    }
}

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
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}",
                scause.cause(),
                stval
            );
        }
    }
    cx
}
