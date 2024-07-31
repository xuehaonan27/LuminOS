use core::arch::asm;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let ret: isize;
    unsafe {
        asm!("ecall",
            // inlateout means x10 serves as both input and output register.
            // {in_var} => {out_var}
            inlateout("x10") args[0] => ret, // a0
            in("x11") args[1], // a1
            in("x12") args[2], // a2
            in("x17") id // a7 ecall id
        );
    }
    ret
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(xstate: i32) -> isize {
    syscall(SYSCALL_EXIT, [xstate as usize, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}
