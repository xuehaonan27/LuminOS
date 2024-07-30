use core::fmt::{self, Write};

use crate::write;

const __STDIN: usize = 0; // fd of standard input
const __STDOUT: usize = 1; // fd of standard output
const __STDERR: usize = 2; // fd of standard error

#[allow(non_camel_case_types)]
struct __stdout;

impl Write for __stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write(__STDOUT, s.as_bytes());
        Ok(())
    }
}

pub fn __print(args: fmt::Arguments) {
    __stdout.write_fmt(args).unwrap()
}

/// Lib print macro
#[macro_export]
macro_rules! print {
    ($fmt: expr $(, $($arg: tt)+)?) => {
        $crate::console::__print(format_args!($fmt $(, $($arg)+)?));
    };
}

/// Lib println macro
#[macro_export]
macro_rules! println {
    ($fmt: expr $(, $($arg: tt)+)?) => {
        $crate::console::__print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
    };
}
