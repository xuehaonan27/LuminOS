#[cfg(feature = "batch")]
use crate::batch::run_next_app;
#[cfg(any(feature = "multiprogramming", feature = "multitasking"))]
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
#[cfg(feature = "multitasking")]
use crate::timer::get_time_us;

/// Batch Kernel: batched app exits and schedule the next one
/// Multiprogramming Kernel: task exits and submit and exit code
pub fn sys_exit(xstate: i32) -> ! {
    kprintln!("[kernel] Application exited with code {}", xstate);
    #[cfg(feature = "batch")]
    run_next_app();
    #[cfg(any(feature = "multiprogramming", feature = "multitasking"))]
    {
        exit_current_and_run_next();
        panic!("Unreachable in sys_exit!");
    }
    #[cfg(not(any(feature = "batch", feature = "multiprogramming", feature = "multitasking")))]
    panic!("Exit with {xstate}");
}

/// Current task gives up resources for other tasks
/// Syscall ID: 124
pub fn sys_yield() -> isize {
    #[cfg(any(feature = "multiprogramming", feature = "multitasking"))]
    suspend_current_and_run_next();
    0
}

/// Get time in microseconds
pub fn sys_get_time() -> isize {
    if cfg!(feature = "multitasking") {
        get_time_us() as isize
    } else {
        0
    }
}
