/// Debug print
#[macro_export]
macro_rules! debug_print {
    ($($args:tt)*) => {
        #[cfg(feature = "debug")]
        kprint!($($args)*);
    };
}

/// Debug println
#[macro_export]
macro_rules! debug_println {
    ($($args:tt)*) => {
        #[cfg(feature = "debug")]
        kprintln!($($args)*);
    };
}