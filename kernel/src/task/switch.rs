use core::arch::global_asm;

#[cfg(feature = "D_EXTENSION_ENABLED")]
global_asm!(include_str!("switch_d_ext.S"));
#[cfg(not(feature = "D_EXTENSION_ENABLED"))]
global_asm!(include_str!("switch.S"));

use super::TaskContext;

extern "C" {
    pub fn __switch(
        current_task_cx_ptr: *mut TaskContext, // a0
        next_task_cx_ptr: *const TaskContext,  // a1
    );
}
