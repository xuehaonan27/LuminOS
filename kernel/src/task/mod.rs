//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;
mod task;

use crate::config::MAX_APP_NUM;
use crate::loader::{get_num_app, init_app_cx};
use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
#[cfg(feature = "profiling")]
use crate::timer::get_time_us;
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

/// Inner of Task Manager
pub struct TaskManagerInner {
    /// task list
    tasks: [TaskControlBlock; MAX_APP_NUM],
    /// id of current `Running` task
    current_task: usize,
    /// calculate kernel space time for a task
    #[cfg(feature = "profiling")]
    stop_watch: usize,
}

impl TaskManagerInner {
    #[cfg(feature = "profiling")]
    fn refresh_stop_watch(&mut self) -> usize {
        let start_time = self.stop_watch;
        self.stop_watch = get_time_us();
        self.stop_watch - start_time
    }
}

lazy_static! {
    /// Global variable: TASK_MANAGER
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
            #[cfg(feature = "profiling")]
            user_time: 0,
            #[cfg(feature = "profiling")]
            kernel_time: 0,
        }; MAX_APP_NUM];
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_cx = TaskContext::goto_restore(init_app_cx(i));
            task.task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                    #[cfg(feature = "profiling")]
                    stop_watch: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch3, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        // start recording
        #[cfg(feature = "profiling")]
        inner.refresh_stop_watch();
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        #[cfg(feature = "profiling")]
        kprintln!("[kernel] task {} exited. user_time: {} us, kernel_time: {} us.", current, inner.tasks[current].user_time, inner.tasks[current].kernel_time);
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            // add kernel time for task A
            // refresh stop watch for recording kernel time of task B
            #[cfg(feature = "profiling")]
            (inner.tasks[current].kernel_time += inner.refresh_stop_watch());
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            // debug_println!("before switch");
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            kprintln!("All applications completed!");
            shutdown(false);
        }
    }
    #[cfg(feature = "profiling")]
    fn user_time_start(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        // add kernel time for task B
        // refresh stop watch for recording user time of task B
        inner.tasks[current].kernel_time += inner.refresh_stop_watch();
    }
    #[cfg(feature = "profiling")]
    fn user_time_end(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        // add user time for task A
        // refresh stop watch for recording kernel time of task A
        inner.tasks[current].user_time += inner.refresh_stop_watch();
    }
}

/// run first task
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// rust next task
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// suspend current task
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// exit current task
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// suspend current task, then run next task
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// exit current task,  then run next task
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}


/// Calcualte kernel time before running in user space
#[cfg(feature = "profiling")]
pub fn user_time_start() {
    TASK_MANAGER.user_time_start()
}

/// Calculate user time before running in kernel space
#[cfg(feature = "profiling")]
pub fn user_time_end() {
    TASK_MANAGER.user_time_end()
}
