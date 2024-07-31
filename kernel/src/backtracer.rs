#![allow(unused)]
use core::arch::asm;

use crate::kprintln;

macro_rules! impl_riscv_xregs {
    ($($x:ident),+) => {
        $(
            #[inline(always)]
            pub fn $x() -> usize {
                let $x: usize;
                unsafe { asm!(concat!("mv {}, ", stringify!($x)), out(reg) $x) };
                $x
            }
        )+
    };
}

macro_rules! impl_riscv_fregs {
    ($($x:ident),+) => {
        $(
            #[inline(always)]
            pub fn $x() -> u64 {
                let $x: u64;
                unsafe { asm!(concat!("fmv.x.w {}, ", stringify!($x)), out(reg) $x) };
                $x
            }
        )+
    };
}

struct RiscvAsm;

impl RiscvAsm {
    /// Program counter
    #[inline(always)]
    pub fn pc() -> usize {
        let pc: usize;
        unsafe { asm!("auipc {0}, 0", out(reg) pc) };
        pc
    }

    /// [`sp`] register
    #[inline(always)]
    pub fn sp() -> usize {
        let sp: usize;
        unsafe { asm!("mv {}, sp", out(reg) sp) };
        sp
    }

    /// [`fp`] register
    #[inline(always)]
    pub fn fp() -> usize {
        let fp: usize;
        unsafe { asm!("mv {}, fp", out(reg) fp) };
        fp
    }

    /// [`gp`] register
    #[inline(always)]
    pub fn gp() -> usize {
        let gp: usize;
        unsafe { asm!("mv {}, gp", out(reg) gp) };
        gp
    }

    /// [`tp`] register
    #[inline(always)]
    pub fn tp() -> usize {
        let tp: usize;
        unsafe { asm!("mv {}, tp", out(reg) tp) };
        tp
    }

    /// [`ra`] register
    #[inline(always)]
    pub fn ra() -> usize {
        let ra: usize;
        unsafe { asm!("mv {}, ra", out(reg) ra) };
        ra
    }

    impl_riscv_xregs!(
        x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15, x16, x17, x18, x19,
        x20, x21, x22, x23, x24, x25, x26, x27, x28, x29, x30, x31
    );

    impl_riscv_fregs!(
        f0, f1, f2, f3, f4, f5, f6, f7, f8, f9, f10, f11, f12, f13, f14, f15, f16, f17, f18, f19,
        f20, f21, f22, f23, f24, f25, f26, f27, f28, f29, f30, f31
    );
}

/*
#[derive(Clone, Debug, Default)]
pub struct Backtrace<const N: usize> {
    pub frames: ArrayVec<usize, N>,

    pub frames_omitted: bool,
}

impl<const N: usize> Backtrace<N> {
    /// Captures a backtrace from the current call point.
    ///
    /// The first frame of the backtrace is the caller of `Backtrace::capture`.
    #[inline(never)]
    pub fn capture() -> Self {
        todo!()
    }
}
*/

#[derive(Copy, Clone, Debug, Default)]
#[allow(unused)]
pub struct BacktraceContext {
    /// Program counter
    pub pc: usize,

    /// General-purpose registers
    pub x1: usize,
    pub x2: usize,
    pub x3: usize,
    pub x4: usize,
    pub x5: usize,
    pub x6: usize,
    pub x7: usize,
    pub x8: usize,
    pub x9: usize,
    pub x10: usize,
    pub x11: usize,
    pub x12: usize,
    pub x13: usize,
    pub x14: usize,
    pub x15: usize,
    pub x16: usize,
    pub x17: usize,
    pub x18: usize,
    pub x19: usize,
    pub x20: usize,
    pub x21: usize,
    pub x22: usize,
    pub x23: usize,
    pub x24: usize,
    pub x25: usize,
    pub x26: usize,
    pub x27: usize,
    pub x28: usize,
    pub x29: usize,
    pub x30: usize,
    pub x31: usize,

    /// Floating-point registers
    pub f0: u64,
    pub f1: u64,
    pub f2: u64,
    pub f3: u64,
    pub f4: u64,
    pub f5: u64,
    pub f6: u64,
    pub f7: u64,
    pub f8: u64,
    pub f9: u64,
    pub f10: u64,
    pub f11: u64,
    pub f12: u64,
    pub f13: u64,
    pub f14: u64,
    pub f15: u64,
    pub f16: u64,
    pub f17: u64,
    pub f18: u64,
    pub f19: u64,
    pub f20: u64,
    pub f21: u64,
    pub f22: u64,
    pub f23: u64,
    pub f24: u64,
    pub f25: u64,
    pub f26: u64,
    pub f27: u64,
    pub f28: u64,
    pub f29: u64,
    pub f30: u64,
    pub f31: u64,
}

#[allow(unused)]
impl BacktraceContext {
    #[inline(always)]
    pub fn snapshot() -> Self {
        let mut cxt = BacktraceContext::default();
        macro_rules! get_regs {
            ($($x:ident),+) => {
                $(
                    cxt.$x = RiscvAsm::$x();
                )+
            };
        }
        get_regs!(
            pc, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15, x16, x17, x18,
            x19, x20, x21, x22, x23, x24, x25, x26, x27, x28, x29, x30, x31, f0, f1, f2, f3, f4,
            f5, f6, f7, f8, f9, f10, f11, f12, f13, f14, f15, f16, f17, f18, f19, f20, f21, f22,
            f23, f24, f25, f26, f27, f28, f29, f30, f31
        );
        cxt
    }

    pub fn pc(&self) -> usize {
        self.pc
    }
}

pub fn test() {
    macro_rules! print_regs {
        ($($x:ident),+) => {
            $(
                kprintln!(
                    concat!(stringify!($x), " = {:#x}"),
                    RiscvAsm::$x()
                );
            )+
        };
    }

    kprintln!("pc = {:#x}", RiscvAsm::pc());
    kprintln!("fp = {:#x}", RiscvAsm::fp());
    kprintln!("sp = {:#x}", RiscvAsm::sp());
    kprintln!("gp = {:#x}", RiscvAsm::gp());
    kprintln!("tp = {:#x}", RiscvAsm::tp());
    kprintln!("ra = {:#x}", RiscvAsm::ra());

    print_regs!(
        x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15, x16, x17, x18, x19,
        x20, x21, x22, x23, x24, x25, x26, x27, x28, x29, x30, x31
    );
}

pub fn backtrace() {
    let mut fp = RiscvAsm::fp();
    let mut ra = RiscvAsm::ra();

    kprintln!("Backtrace:");
    while fp != 0 {
        kprintln!("FP: {:#x}, RA: {:#x}", fp, ra);

        // stack layout:
        // [parent stack frame]
        // >-----fp-----<
        // [return address (ra)]
        // [saved frame pointer (s0/fp)]
        // [callee-saved registers]
        // [local variables]
        // >-----sp-----<
        unsafe {
            // get return address of current frame
            ra = *(fp as *const usize).offset(-1);
            // get frame pointer to parent frame
            fp = *(fp as *const usize).offset(-2);
        }
    }
    kprintln!("FP: {:#x}, RA: {:#x}", fp, ra);
}
