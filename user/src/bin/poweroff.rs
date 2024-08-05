#![no_std]
#![no_main]

use user_lib::reboot;

extern crate user_lib;

#[no_mangle]
pub fn main() -> ! {
    reboot()
}