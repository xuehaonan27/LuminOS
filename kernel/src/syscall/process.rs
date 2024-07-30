#[cfg(feature = "batch")]
use crate::batch::run_next_app;

pub fn sys_exit(xstate: i32) -> ! {
    kprintln!("[kernel] Application exited with code {}", xstate);
    #[cfg(feature = "batch")]
    run_next_app();
    #[cfg(not(feature = "batch"))]
    panic!("Exit with {xstate}")
}