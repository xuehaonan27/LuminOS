mod fs;
mod process;
use fs::*;
use process::*;

const __SYSCALL_WRITE: usize = 64;
const __SYSCALL_EXIT: usize = 93;
const __SYSCALL_YIELD: usize = 124;
const __SYSCALL_GET_TIME: usize = 169;

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        __SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        __SYSCALL_EXIT => sys_exit(args[0] as i32),
        __SYSCALL_YIELD => sys_yield(),
        __SYSCALL_GET_TIME => sys_get_time(),
        _ => panic!("Unsupported syscall id: {}", id),
    }
}
