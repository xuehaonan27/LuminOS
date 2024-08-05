//! Constants

pub const USER_STACK_SIZE: usize = 4096 * 2; // 8 KiB
pub const KERNEL_STACK_SIZE: usize = 4096 * 2; // 8 KiB
pub const CLOCK_FREQ: usize = 12500000; // On QEMU
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const POINTER_SIZE: usize = core::mem::size_of::<usize>() * 8;
pub const PAGE_SIZE_BITS: usize = 0xc; // 4KiB
pub const PAGE_SIZE: usize = 0x1000; // 4KiB
pub const MEMORY_END: usize = 0x8800_0000; // Physical memory 8MiB
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1; // Top most page in virtual space
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE; // Second top most page in virtual space
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
