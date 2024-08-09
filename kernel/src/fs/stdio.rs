use super::File;
use crate::mm::UserBuffer;
use crate::sbi::console_getchar;
use crate::task::suspend_current_and_run_next;

/// Standard input
#[derive(Debug)]
pub struct Stdin;
/// Standard output
#[derive(Debug)]
pub struct Stdout;
/// Standard error
#[derive(Debug)]
pub struct Stderr;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        assert_eq!(buf.len(), 1);
        // busy loop
        let mut c: usize;
        loop {
            c = console_getchar();
            if c == 0 {
                suspend_current_and_run_next();
                continue;
            } else {
                break;
            }
        }
        let ch = c as u8;
        unsafe {
            buf.buffers[0].as_mut_ptr().write_volatile(ch);
        }
        1
    }
    fn write(&self, _buf: UserBuffer) -> usize {
        panic!("Cannot write to stdin!"); // FIXME: better error handling
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, _buf: UserBuffer) -> usize {
        panic!("Cannot read from stdout!"); // FIXME: better error handling
    }
    fn write(&self, buf: UserBuffer) -> usize {
        for buffer in buf.buffers.iter() {
            kprint!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        buf.len()
    }
}

impl File for Stderr {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, _buf: UserBuffer) -> usize {
        panic!("Cannot read from stderr!"); // FIXME: better error handling
    }
    fn write(&self, buf: UserBuffer) -> usize {
        for buffer in buf.buffers.iter() {
            kprint!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        buf.len()
    }
}
