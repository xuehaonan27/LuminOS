use riscv::register::sstatus;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TrapContext {
    /// General purpose registers
    pub x: [usize; 32],

    /// S Mode sstatus register
    pub sstatus: usize,

    /// S Mode sepc register
    pub sepc: usize,

    /// Float point registers
    #[cfg(feature = "D_EXTENSION_ENABLED")]
    pub f: [u64; 32],

    /// Kernel page table address
    pub kernel_satp: usize,

    /// Kernel stack top for this process
    pub kernel_sp: usize,

    /// Trap handler
    pub trap_handler: usize,
}

#[allow(unused)]
impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    pub fn init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let sstatus = sstatus::read();
        let mut sstatus: usize = unsafe { core::mem::transmute(sstatus) };
        // set SPP to user mode
        sstatus &= !(1 << 8);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
            #[cfg(feature = "D_EXTENSION_ENABLED")]
            f: [0; 32],
            kernel_satp,
            kernel_sp,
            trap_handler,
        };
        cx.set_sp(sp);
        cx
    }
}
