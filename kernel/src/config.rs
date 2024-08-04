//! Constants

pub const USER_STACK_SIZE: usize = 4096 * 2; // 8 KiB
pub const KERNEL_STACK_SIZE: usize = 4096 * 2; // 8 KiB
#[cfg(not(feature = "vmm"))]
pub const MAX_APP_NUM: usize = 16;
#[cfg(not(feature = "vmm"))]
pub const APP_BASE_ADDRESS: usize = 0x80400000;
#[cfg(not(feature = "vmm"))]
pub const APP_SIZE_LIMIT: usize = 0x20000;

pub const CLOCK_FREQ: usize = 12500000; // On QEMU

#[cfg(feature = "vmm")]
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
#[cfg(feature = "vmm")]
pub const POINTER_SIZE: usize = core::mem::size_of::<usize>() * 8;
#[cfg(feature = "vmm")]
pub const PAGE_SIZE_BITS: usize = 0xc; // 4KiB
#[cfg(feature = "vmm")]
pub const PAGE_SIZE: usize = 0x1000; // 4KiB
#[cfg(feature = "vmm")]
pub const MEMORY_END: usize = 0x80800000; // Physical memory 8MiB
#[cfg(feature = "vmm")]
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1; // Top most page in virtual space
#[cfg(feature = "vmm")]
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE; // Second top most page in virtual space
#[cfg(feature = "vmm")]
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}