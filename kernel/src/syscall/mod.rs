mod fs;
mod process;
use fs::sys_write;
use process::sys_exit;

const __SYSCALL_WRITE: usize = 64;
const __SYSCALL_EXIT: usize = 93;

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        __SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        __SYSCALL_EXIT => sys_exit(args[0] as i32),
        _ => panic!("Unsupported syscall id: {}", id),
    }
}
