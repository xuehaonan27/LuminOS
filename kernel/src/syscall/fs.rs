const __STDOUT: usize = 1;
const __STDERR: usize = 2;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        __STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            kprint!("{}", str);
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
