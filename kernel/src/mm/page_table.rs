use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;

use super::{
    address::{PhysPageNum, StepByOne, VirtPageNum, PPN_WIDTH_SV39},
    frame_allocator::{frame_alloc, FrameTracker},
    VirtAddr,
};

const PTE_PPN_OFFSET: usize = 10;

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct PTEFlags: u8 {
        const V = 1 << 0; // Valid
        const R = 1 << 1; // Read
        const W = 1 << 2; // Write
        const X = 1 << 3; // Execute
        const U = 1 << 4; // User mode access
        const G = 1 << 5; // Global
        const A = 1 << 6; // Access (set by hardware)
        const D = 1 << 7; // Dirty (set by hardware)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    /// PTE from PPN and flags
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << PTE_PPN_OFFSET | flags.bits() as usize,
        }
    }
    /// Empty PTE, which implies this PTE is invalid
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    /// PPN field of PTE
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> PTE_PPN_OFFSET & ((1usize << PPN_WIDTH_SV39) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

/// Page table structure
#[derive(Debug)]
pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>, // RAII saving FrameTracker
                               // FIXME: should we holds anothor instance of FrameTracker in PageTable?
                               // We have already had memory area to do this.
                               // Conflict-prone!
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            if i == 2 {
                // 3 level page table
                result = Some(pte);
                break;
            }
            // 1 level and 2 level page table
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap(); // allocate a frame for new page table
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V); // valid, but rwx = 000, indicating this is a PTE points to another page table instead of page.
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            if i == 2 {
                // result maybe an invalid PTE.
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn: {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }
    /// Temporarily used to get arguments from user space.
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << PPN_WIDTH_SV39) - 1)),
            frames: Vec::new(), // frames field set empty to avoid holding and resource.
        }
    }
    /// Get corresponding page table entry of `vpn`
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| pte.clone())
    }
    /// Token for SATP register
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}

pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token); // temporary page table, does not hold resources
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()])
        }
        start = end_va.into();
    }
    v
}
