use crate::memory::{
    PAGE_SIZE,
    PhysicalAddress,
    PhysicalPage,
    PhysicalRange,
};
use crate::sync::RwLock;
use crate::x86;
use core::fmt;

/// PageRef is designed to resemble a Rust enum, but isn't one to ensure it
/// fits in a single byte. It does this by having a limited range, supporting
/// values from 0..252 and using the other representable values for the
/// cases where there is no memory, the refcount is exceeded, or etc.
#[derive(Copy, Clone, PartialEq, Eq)]
struct PageRef(u8);

#[allow(non_upper_case_globals)]
impl PageRef {
    const NO_MEMORY: u8 = 0;
    const LEAK: u8 = 1;
    const ZERO: u8 = 2;

    const NoMemory: PageRef = PageRef(0);
    const Leak: PageRef = PageRef(1);
    const Zero: PageRef = PageRef(2);

    fn from_multiboot(mb_type: multiboot2::MemoryAreaType) -> Self {
        match mb_type {
            multiboot2::MemoryAreaType::Available => PageRef::Zero,
            _ => PageRef::Leak,
        }
    }

    fn is_usable(&self) -> bool {
        self.0 == PageRef::ZERO
    }

    fn in_use(&self) -> bool {
        self.0 != PageRef::ZERO && self.0 != PageRef::NO_MEMORY
    }

    fn has_references(&self) -> bool {
        self.0 > PageRef::ZERO
    }

    fn is_counted(&self) -> bool {
        self.0 >= PageRef::ZERO
    }

    fn incref(&mut self) {
        if self.0 == u8::MAX {
            self.0 = PageRef::LEAK;
        }
        if self.is_counted() {
            self.0 += 1;
        }
    }

    fn decref(&mut self) {
        if self.has_references() {
            self.0 -= 1;
        }
    }

    fn count(&self) -> Option<usize> {
        if self.is_counted() {
            Some((self.0 - PageRef::ZERO) as usize)
        } else {
            None
        }
    }
}

impl fmt::Debug for PageRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(count) = self.count() {
            f.debug_tuple("PageRef").field(&count).finish()
        } else {
            match *self {
                PageRef::NoMemory => f.write_str("NoMemory"),
                PageRef::Leak => f.write_str("Leak"),
                _ => panic!("unreachable"),
            }
        }
    }
}

struct PhysicalMap {
    map: [PageRef; PhysicalMap::PAGE_COUNT],
}

impl PhysicalMap {
    const PAGE_COUNT: usize = 0x4000;

    fn new() -> Self {
        Self {
            map: [PageRef::NoMemory; PhysicalMap::PAGE_COUNT],
        }
    }

    fn set_index(&mut self, index: usize, v: PageRef) {
        if index >= Self::PAGE_COUNT {
            return;
        }

        let current = self.map[index];

        if current == PageRef::NoMemory || v == PageRef::Leak {
            self.map[index] = v;
        }
    }

    fn set_index_range(&mut self, r: impl Iterator<Item = PhysicalPage>, v: PageRef) {
        for p in r {
            self.set_index(p.index(), v);
        }
    }

    fn set(&mut self, p: PhysicalAddress, v: PageRef) {
        self.set_index(p.page().index(), v)
    }

    fn set_range(&mut self, p: PhysicalRange, v: PageRef) {
        self.set_index_range(p.pages(), v)
    }

    fn incref(&mut self, p: PhysicalAddress) {
        self.map[p.page().index()].incref()
    }

    fn decref(&mut self, p: PhysicalAddress) {
        self.map[p.page().index()].decref()
    }

    fn usable_index(&self) -> Option<usize> {
        for (i, r) in self.map.iter().enumerate() {
            if r.is_usable() {
                return Some(i);
            }
        }
        None
    }

    fn alloc(&mut self) -> Option<PhysicalAddress> {
        if let Some(i) = self.usable_index() {
            let page = PhysicalAddress(i * PAGE_SIZE);
            self.incref(page);
            Some(page)
        } else {
            None
        }
    }

    fn free(&mut self, p: PhysicalAddress) {
        self.decref(p);
    }

    fn summarize(&self) {
        let mut in_use = 0;
        let mut available = 0;
        let mut leaked = 0;

        for (_i, r) in self.map.iter().enumerate() {
            if r.in_use() {
                in_use += 0x1000;
            } else if r.is_usable() {
                available += 0x1000;
            } else if *r == PageRef::Leak {
                leaked += 0x1000;
            }
        }

        println!(
            "in_use: {:x} available: {:x} leaked: {:x}",
            in_use, available, leaked
        );
    }
}

lazy_static! {
    static ref PHYSICAL_MEMORY_MAP: RwLock<PhysicalMap> =
        RwLock::new(PhysicalMap::new());
}

pub fn map_init(areas: multiboot2::MemoryAreaIter<'_>) {
    for area in areas {
        let range = PhysicalRange::from_multiboot_area(area);
        let r = PageRef::from_multiboot(area.typ());

        println!(
            "memory map: {:>10x} {:>10x} {:?}",
            area.start_address(),
            area.size(),
            r
        );

        PHYSICAL_MEMORY_MAP.write().set_range(range, r);
    }

    let kernel_range = PhysicalRange {
        start: x86::kernel_start(),
        end: x86::kernel_end(),
    };

    println!("Leaking kernel: {:x?}", kernel_range);

    PHYSICAL_MEMORY_MAP
        .write()
        .set_range(kernel_range, PageRef::Leak);
}

pub fn leak(r: PhysicalRange) {
    PHYSICAL_MEMORY_MAP.write().set_range(r, PageRef::Leak);
}

pub fn alloc() -> PhysicalAddress {
    PHYSICAL_MEMORY_MAP.write().alloc().expect("Out of memory")
}

pub fn alloc_zero() -> PhysicalAddress {
    let page = PHYSICAL_MEMORY_MAP.write().alloc().expect("Out of memory");
    unsafe { page.write_phy([0; PAGE_SIZE]) };
    page
}

pub fn free(p: PhysicalAddress) {
    PHYSICAL_MEMORY_MAP.write().free(p)
}
