use core::mem::size_of;
use core::ops::{Add, BitAnd, BitOr, Range};
use crate::PHY_OFFSET;
use crate::util::round_down;
use super::PAGE_SIZE;
use super::phy;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PhysicalAddress(pub usize);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PhysicalPage(pub usize);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct PhysicalRange(pub usize, pub usize);

#[derive(Debug)]
pub struct PageTable(pub PhysicalPage);

#[derive(Copy, Clone, PartialEq, Eq,Debug)]
#[repr(C)]
pub struct PageTableEntry(usize);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct VirtualAddress(pub usize);


#[derive(Copy, Clone, Debug)]
enum PagingError<'a> {
    Other(&'a str),
}


const PAGE_MASK: usize = 0xFFFF_FFFF_FFFF_F000;
const PAGE_OFFSET_MASK: usize = !PAGE_MASK;
const PAGE_ADDR_MASK: usize = 0x00FF_FFFF_FFFF_F000;
const PAGE_FLAGS_MASK: usize = 0xFF00_0000_0000_0FFF;

const PAGE_PRESENT: usize = 0x01;
const PAGE_WRITEABLE: usize = 0x02;
const PAGE_USERMODE: usize = 0x04;
const PAGE_ACCESSED: usize = 0x20;
const PAGE_DIRTY: usize = 0x40;
const PAGE_ISHUGE: usize = 0x80;
const PAGE_GLOBAL: usize = 0x100;
const PAGE_OS_RESERVED1: usize = 0x200;
const PAGE_OS_RESERVED2: usize = 0x400;
const PAGE_OS_RESERVED3: usize = 0x800;

const PAGE_TABLE_FLAGS: usize = PAGE_PRESENT | PAGE_WRITEABLE;
const PAGE_COPYONWRITE: usize = PAGE_OS_RESERVED1;
const PAGE_UNBACKED: usize = 0x1000000;


impl PhysicalAddress {
    fn page(self) -> PhysicalPage {
        PhysicalPage(self.0 & PAGE_MASK)
    }

    fn page_offset(self) -> usize {
        self.0 & PAGE_OFFSET_MASK
    }

    pub unsafe fn read_phy<T: Copy>(self) -> T {
        *((self.0 + PHY_OFFSET) as *const T)
    }

    pub unsafe fn write_phy<T>(self, v: T) {
        *((self.0 + PHY_OFFSET) as *mut T) = v
    }

    pub unsafe fn as_ref<T>(self) -> &'static T {
        &*((self.0 + PHY_OFFSET) as *const T)
    }

    pub unsafe fn as_mut<T>(self) -> &'static mut T {
        &mut *((self.0 + PHY_OFFSET) as *mut T)
    }
}

impl Add<usize> for PhysicalAddress {
    type Output = Self;
    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs)
    }
}


impl PhysicalPage {
    fn base_address(self) -> PhysicalAddress {
        PhysicalAddress(self.0)
    }

    fn index(self) -> usize {
        self.0 / PAGE_SIZE
    }
}


struct PhysicalRangeIter {
    base: usize,
    top: usize,
    cursor: usize,
}

impl Iterator for PhysicalRangeIter {
    type Item = PhysicalAddress;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor > self.top {
            None
        } else {
            self.cursor += 1;
            Some(PhysicalAddress(self.cursor))
        }
    }
}

struct PhysicalRangePages {
    base: usize,
    top: usize,
    cursor: usize,
}

impl Iterator for PhysicalRangePages {
    type Item = PhysicalPage;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor > self.top {
            None
        } else {
            self.cursor += PAGE_SIZE;
            Some(PhysicalPage(self.cursor))
        }
    }
}

impl PhysicalRange {
    fn from_range(r: Range<usize>) -> Self {
        Self(r.start, r.end)
    }

    fn iter(self) -> impl Iterator<Item = PhysicalAddress> {
        PhysicalRangeIter { base: self.0, top: self.1, cursor: self.0 }
    }

    fn pages(self) -> impl Iterator<Item = PhysicalPage> {
        let base = round_down(self.0, PAGE_SIZE);
        PhysicalRangePages { base, top: self.1, cursor: base }
    }
}


impl PageTable {
    fn entry_address(root: PhysicalPage, index: usize) -> PhysicalAddress {
        root.base_address() + index * size_of::<usize>()
    }

    fn entry(root: PhysicalPage, index: usize) -> PageTableEntry {
        unsafe { Self::entry_address(root, index).read_phy() }
    }

    fn entry_mut(root: PhysicalPage, index: usize) -> &'static mut PageTableEntry {
        unsafe { Self::entry_address(root, index).as_mut() }
    }

    fn make_next_table(p: &mut PageTableEntry, kernel: bool) {
        let flags = if kernel {
            PAGE_TABLE_FLAGS
        } else {
            PAGE_TABLE_FLAGS | PAGE_USERMODE
        };
        *p = PageTableEntry(phy::alloc_zero().0 | flags);
    }

    fn offset(v: VirtualAddress, level: usize) -> usize {
        (v.0 >> (12 + level * 9)) & 0xFFF
    }

    fn pte_mut_recursive(
        &self,
        root: PhysicalPage,
        v: VirtualAddress,
        level: usize,
        create: bool,
    ) -> Result<&mut PageTableEntry, PagingError> {
        let offset = Self::offset(v, level);
        let entry = Self::entry_mut(root, offset);
        if level == 1 {
            return Ok(entry);
        }
        if !entry.present() {
            if create {
                Self::make_next_table(entry, v.is_higher_half());
            } else {
                return Err(PagingError::Other("Page Not Present"));
            }
        }
        self.pte_mut_recursive(entry.deref(), v, level - 1, create)
    }

    fn pte(&self, v: VirtualAddress) -> PageTableEntry {
        self.pte_mut_recursive(self.0, v, 4, false)
            .map(|p| *p)
            .unwrap_or(PageTableEntry::nil())
    }

    fn pte_mut(&mut self, v: VirtualAddress) -> &mut PageTableEntry {
        &mut *self.pte_mut_recursive(self.0, v, 4, true).unwrap()
    }

    fn map(&mut self, v: VirtualAddress, p: PhysicalPage, flags: usize) {
        *self.pte_mut(v) = PageTableEntry::from_page_flags(p, flags);
    }

    fn unmap(&mut self, v: VirtualAddress) {
        *self.pte_mut(v) = PageTableEntry::nil();
    }
}


impl PageTableEntry {
    fn from_page_flags(p: PhysicalPage, f: usize) -> Self {
        Self(p.0 | f)
    }

    fn nil() -> Self {
        Self(0)
    }

    fn deref(self) -> PhysicalPage {
        PhysicalPage(self.0 & PAGE_ADDR_MASK)
    }

    fn present(self) -> bool {
        self.0 & PAGE_PRESENT != 0
    }

    // writeable(), usermode(), etc are harder to do correctly, since
    // in the actual hardware they depend on the values in pages above
    // them to set the actual value used by hardware.
}

impl BitOr<usize> for PageTableEntry {
    type Output = Self;
    fn bitor(self, rhs: usize) -> Self {
        Self(self.0 | rhs)
    }
}

impl BitAnd<usize> for PageTableEntry {
    type Output = Self;
    fn bitand(self, rhs: usize) -> Self {
        Self(self.0 & rhs)
    }
}

impl VirtualAddress {
    fn is_higher_half(self) -> bool {
        self.0 > 0x8000_0000_0000
    }
}
