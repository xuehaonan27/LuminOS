use crate::{mm::translated_byte_buffer, task::current_user_token};

const __STDOUT: usize = 1;
const __STDERR: usize = 2;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        #[cfg(not(feature = "vmm"))]
        __STDOUT => {
            // Legacy code
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            kprint!("{}", str);
            len as isize
        }
        #[cfg(feature = "vmm")]
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
