use super::TaskContext;

#[derive(Clone, Copy, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

#[derive(Clone, Copy)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    #[cfg(feature = "profiling")]
    /// Total time running in user space (in us)
    pub user_time: usize,
    #[cfg(feature = "profiling")]
    /// Total time running in kernel space (in us)
    pub kernel_time: usize,
}
