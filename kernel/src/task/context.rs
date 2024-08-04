#[cfg(feature = "vmm")]
use crate::trap::trap_return;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    /// return address ( e.g. __restore ) of __switch ASM function
    ra: usize,
    /// kernel stack pointer of app
    sp: usize,
    /// callee saved registers:  s 0..11
    s: [usize; 12],
    #[cfg(feature = "D_EXTENSION_ENABLED")]
    /// callee saved fload registers: f 0..11
    fs: [u64; 12],
}

impl TaskContext {
    /// init task context
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
            #[cfg(feature = "D_EXTENSION_ENABLED")]
            fs: [0; 12],
        }
    }

    /// set task context {__restore ASM funciton, kernel stack, s_0..12 }
    #[cfg(not(feature = "vmm"))]
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize,
            sp: kstack_ptr,
            s: [0; 12],
            #[cfg(feature = "D_EXTENSION_ENABLED")]
            fs: [0; 12],
        }
    }

    #[cfg(feature = "vmm")]
    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}
