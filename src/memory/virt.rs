// use core::ops::Range;
// use super::PAGE_SIZE;
// use super::phy::{self, PhysicalAddress};

#[derive(Copy, Clone)]
pub struct VirtualAddress(pub usize);

impl VirtualAddress {
    fn page_offset(self) -> usize {
        self.0 & 0xFFF
    }
}

#[derive(Copy, Clone)]
struct PageTableEntry(usize);

impl PageTableEntry {
    const PRESENT: usize = 0x01;
    const WRITEABLE: usize = 0x02;
    const USERMODE: usize = 0x04;
    const ACCESSED: usize = 0x20;
    const DIRTY: usize = 0x40;
    const ISHUGE: usize = 0x80;
    const GLOBAL: usize = 0x100;

    const OS_RESERVED1: usize = 0x200;
    const OS_RESERVED2: usize = 0x400;
    const OS_RESERVED3: usize = 0x800;

    const COPYONWRITE: usize = Self::OS_RESERVED1;
    const UNBACKED: usize = 0x1000000;

    const PAGE_ADDR_MASK: usize = 0x00FF_FFFF_FFFF_F000;
    const PAGE_FLAGS_MASK: usize = 0xFF00_0000_0000_0FFF;

    fn page(self) -> usize {
        self.0 & Self::PAGE_ADDR_MASK
    }
}

const PAGE_OFFSET_4K: usize = 0xFFF;
const PAGE_MASK_4K: usize = !PAGE_OFFSET_4K;
const PAGE_FLAGS_MASK: usize = 0xFF00000000000FFF;
const PAGE_ADDR_MASK: usize = 0x00FFFFFFFFFFF000;

// ======================================================================= //

// enum PagingError {
//     Other(&'static str),
// }
//
// struct PageTable {
//     root: PhysicalAddress,
// }
//
// impl PageTable {
//     pub fn new() -> Self {
//         Self {
//             root: phy::alloc_zero(),
//         }
//     }
//
//     pub fn fork() -> Self {
//         todo!();
//     }
//
//     pub fn pte_mut(&mut self, v: VirtualAddress) -> Result<&mut PageTableEntry, PagingError> {
//         self.pte_mut_recursive(self.root, v, 4, false)
//     }
//
//     pub fn pte(&self, v: VirtualAddress) -> Option<PageTableEntry> {
//         self.pte_mut_recursive(self.root, v, 4, false).ok().cloned()
//     }
//
//     pub fn resolve(&self, v: VirtualAddress) -> Option<PhysicalAddress> {
//         self.pte(v).map(|pte| PhysicalAddress(pte.page() | v.page_offset()))
//     }
//
//     pub fn map(&mut self, v: VirtualAddress, p: PhysicalAddress, flags: usize) -> Result<(), PagingError> {
//         self.pte_mut(v).map(|pte| { *pte = PageTableEntry(p.0 | flags); })
//     }
//
//     pub fn map_range(&mut self, v: VirtualAddress, p: Range<PhysicalAddress>) -> Result<(), PagingError> {
//         for (i, page) in p.enumerate() {
//             self.map(v + i * PAGE_SIZE, page)?
//         }
//     }
//
//     pub fn unmap(&mut self, v: VirtualAddress) -> Result<(), PagingError> {
//         self.map(v, 0, 0)
//     }
//
//     pub fn unmap_range(&mut self, v: Range<VirtualAddress>) -> Result<(), PagingError> {
//         for page in v {
//             self.unmap(page)?
//         }
//     }
//
//     fn offset(v: VirtualAddress, level: usize) -> usize {
//         (v.0 >> (12 + level * 9)) & 0xFFF;
//     }
//
//     fn pte_mut_recursive(&self, root: PhysicalAddress, v: VirtualAddress, level: usize, create: bool) -> Result<&mut PageTableEntry, PagingError> {
//         // NB: `self` is only here for the lifetime - it's not to be
//         // used in this method.
//         let offset = Self::offset(v, level);
//         let entry = &mut root[offset];
//         if level == 1 {
//             return Ok(entry);
//         }
//         if !(root[offset] & PageTableEntry::PRESENT) {
//             if create {
//                 Self::make_next_table(entry, v.is_higher_half());
//             } else {
//                 return Err(PagingError::Other("Page Not Present"));
//             }
//         }
//         self.pte_mut_recursive(entry.page(), v, level-1, create)
//     }
//
//     fn make_next_table(p: &mut PageTableEntry, kernel: bool) {
//         let flags = if kernel {
//             PageTableEntry::TABLE_FLAGS
//         } else {
//             PageTableEntry::TABLE_FLAGS | PageTableEntry::USERMODE
//         };
//         *p = phy::alloc_zero() | flags;
//     }
//
//     fn pte_mut_create(&mut self, v: VirtualAddress) -> &mut PageTableEntry {
//         self.pte_mut_recursive(self.root, v, 4, true).expect("pte_mut_recursive failed")
//     }
// }
