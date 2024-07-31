use riscv::register::time;

use crate::{config::CLOCK_FREQ, sbi::set_timer};

const TICKS_PER_SEC: usize = 100; // Timer interrupt every 10ms.
const MICRO_PER_SEC: usize = 1_000_000; // 1 millon microseconds per second.

/// Get value from `mtime` register.
pub fn get_time() -> usize {
    time::read()
}

/// Set when the next timer interrupt should occur.
/// Set value to `mtimecmp` register.
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}

/// Get time in microseconds.
pub fn get_time_us() -> usize {
    time::read() / (CLOCK_FREQ / MICRO_PER_SEC)
}