use core::fmt::{self, Write};

use crate::sbi::console_putchar;

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
