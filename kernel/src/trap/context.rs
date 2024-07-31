use riscv::register::sstatus;

#[repr(C)]
pub struct TrapContext {
    /// General purpose registers
    pub x: [usize; 32],

    /// S Mode sstatus register
    pub sstatus: usize,

    /// S Mode sepc register
    pub sepc: usize,

    /// Float point registers
    pub f: [u64; 32],
}

#[allow(unused)]
impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    pub fn init_context(entry: usize, sp: usize) -> Self {
        let sstatus = sstatus::read();
        let mut sstatus: usize = unsafe { core::mem::transmute(sstatus) };
        // set SPP to user mode
        sstatus &= !(1 << 8);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
            f: [0; 32],
        };
        cx.set_sp(sp);
        cx
    }
}
