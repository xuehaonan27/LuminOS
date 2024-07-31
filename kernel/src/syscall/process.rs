#[cfg(feature = "batch")]
use crate::batch::run_next_app;
#[cfg(feature = "multiprogramming")]
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};

/// Batch Kernel: batched app exits and schedule the next one
/// Multiprogramming Kernel: task exits and submit and exit code
pub fn sys_exit(xstate: i32) -> ! {
    kprintln!("[kernel] Application exited with code {}", xstate);
    #[cfg(feature = "batch")]
    run_next_app();
    #[cfg(feature = "multiprogramming")]
    {
        exit_current_and_run_next();
        panic!("Unreachable in sys_exit!");
    }
    #[cfg(not(any(feature = "batch", feature = "multiprogramming")))]
    panic!("Exit with {xstate}");
}

/// Current task gives up resources for other tasks
/// Syscall ID: 124
pub fn sys_yield() -> isize {
    #[cfg(feature = "multiprogramming")]
    suspend_current_and_run_next();
    0
}
