use crate::{
    mm::translated_byte_buffer,
    sbi::console_getchar,
    task::{current_user_token, suspend_current_and_run_next},
};

const __STDIN: usize = 0;
const __STDOUT: usize = 1;
const __STDERR: usize = 2;

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        __STDIN => {
            assert_eq!(len, 1, "Only support len = 1 in sys_read!");
            let mut c: usize;
            loop {
                c = console_getchar();
                if c == 0 {
                    suspend_current_and_run_next(); // wait
                    continue;
                } else {
                    break;
                }
            }
            let ch = c as u8;
            let mut buffers = translated_byte_buffer(current_user_token(), buf, len);
            unsafe { buffers[0].as_mut_ptr().write_volatile(ch) };
            1
        }
        _ => {
            panic!("Unsupported fd in sys_read");
        }
    }
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        __STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                kprint!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        __STDERR => {
            unimplemented!();
        }
        _ => {
            panic!("Unsupported fd in sys_write");
        }
    }
}
