use core::fmt::{self, Write};

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

struct Kout;

impl Write for Kout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Kout.write_fmt(args).unwrap()
}

/// Kernel print macro
#[macro_export]
macro_rules! kprint {
    ($fmt: expr $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    };
}

/// Kernel println macro
#[macro_export]
macro_rules! kprintln {
    ($fmt: expr $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
    };
}

/// Debug print
#[macro_export]
macro_rules! debug_print {
    ($($args:tt)*) => {
        kprint!($($args)*);
    };
}

/// Debug println
#[macro_export]
macro_rules! debug_println {
    ($($args:tt)*) => {
        kprintln!($($args)*);
    };
}