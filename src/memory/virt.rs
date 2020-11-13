const PAGE_SIZE: usize = 0x1000;

pub struct VirtualAddress(pub usize);

impl VirtualAddress {}

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
}

const PAGE_OFFSET_4K: usize = 0xFFF;
const PAGE_MASK_4K: usize = !PAGE_OFFSET_4K;
const PAGE_FLAGS_MASK: usize = 0xFF00000000000FFF;
const PAGE_ADDR_MASK: usize = 0x00FFFFFFFFFFF000;

//
// enum vmm_copy_op {
//         COPY_COW,
//         COPY_SHARED,
//         COPY_EAGER,
// };
//
// phys_addr_t vmm_resolve(virt_addr_t vma);
// phys_addr_t vmm_virt_to_phy(virt_addr_t vma);
// uintptr_t *vmm_pte_ptr(virt_addr_t vma);
//
// bool vmm_map(virt_addr_t vma, phys_addr_t pma, int flags);
// void vmm_map_range(virt_addr_t vma, phys_addr_t pma, size_t len, int flags);
// void vmm_create_unbacked(virt_addr_t vma, int flags);
// void vmm_create_unbacked_range(virt_addr_t vma, size_t len, int flags);
// bool vmm_unmap(virt_addr_t vma);
// void vmm_unmap_range(virt_addr_t vma, size_t len);
//
// void vmm_copy(virt_addr_t vma, phys_addr_t new_root, enum vmm_copy_op op);
// void vmm_copy_region(virt_addr_t base, virt_addr_t top, phys_addr_t new_root, enum vmm_copy_op op);
//
// phys_addr_t vmm_fork(struct process *);
// void vmm_destroy_tree(phys_addr_t root);
//
// void vmm_early_init(void);
//
// enum fault_result vmm_do_page_fault(virt_addr_t fault_addr, enum x86_fault reason);
//
// #define VMM_MAP_BASE 0xFFFF800000000000
//
// static size_t vm_offset(virt_addr_t vma, int level) {
//         assert(level > 0 && level < 5);
//         return (vma >> (12 + 9 * (level-1))) & 0777;
// }
//
// static bool is_unbacked(uintptr_t pte) {
//         return (!(pte & PAGE_PRESENT) && (pte & PAGE_UNBACKED));
// }
//
// void reset_tlb() {
//         uintptr_t cr3;
//         asm volatile("mov %%cr3, %0" : "=a"(cr3));
//         asm volatile("mov %0, %%cr3" ::"a"(cr3));
// }
//
// uintptr_t __make_next_table(uintptr_t *pte_ptr, bool kernel) {
//         phys_addr_t next_table = pm_alloc();
//         memset((void *)(next_table + VMM_MAP_BASE), 0, PAGE_SIZE);
//         uintptr_t next_pte = next_table | PAGE_PRESENT | PAGE_WRITEABLE;
//         if (!kernel) {
//                 next_pte |= PAGE_USERMODE;
//         }
//         *pte_ptr = next_pte;
//         return next_pte;
// }
//
// uintptr_t *__vmm_pte_ptr(virt_addr_t vma, phys_addr_t root, int level, int create) {
//         size_t offset = vm_offset(vma, level);
//         // printf("vma: %p root: %p level: %i off: %03x\n", vma, root, level, offset);
//         uintptr_t *table = (uintptr_t *)(root + VMM_MAP_BASE);
//         uintptr_t pte = table[offset];
//         if (level == 1) {
//                 return &table[offset];
//         }
//         // printf("addr: %p pte: %p\n", &table[offset], pte);
//
//         if (!(pte & PAGE_PRESENT)) {
//                 if (create) {
//                         pte = __make_next_table(&table[offset], vma > 0xFFFF000000000000);
//                         // printf("new table for %p : pte %p\n", vma, pte);
//                 } else {
//                         return NULL;
//                 }
//         }
//         assert(!(pte & PAGE_ISHUGE)); // no support at this time
//         return __vmm_pte_ptr(vma, pte & PAGE_ADDR_MASK, level-1, create);
// }
//
// phys_addr_t vmm_virt_to_phy(virt_addr_t vma) {
//         phys_addr_t vm_root = running_process->vm_root;
//         uintptr_t *pte_ptr = __vmm_pte_ptr(vma, vm_root, 4, 0);
//         if (!pte_ptr)  return -1;
//         uintptr_t pte = *pte_ptr;
//         return (pte & PAGE_ADDR_MASK) + (vma & PAGE_OFFSET_4K);
// }
//
// uintptr_t *vmm_pte_ptr(virt_addr_t vma) {
//         phys_addr_t vm_root = running_process->vm_root;
//         return __vmm_pte_ptr(vma, vm_root, 4, 0);
// }
//
// bool __vmm_map(virt_addr_t vma, phys_addr_t pma, int flags, bool force) {
//         phys_addr_t vm_root = running_process->vm_root;
//         uintptr_t *pte_ptr = __vmm_pte_ptr(vma, vm_root, 4, 1);
//         uintptr_t old_page = *pte_ptr & PAGE_ADDR_MASK;
//         if (!pte_ptr)  return false;
//         if (*pte_ptr && !force)  return false;
//
//         *pte_ptr = (pma & PAGE_MASK_4K) | flags;
//         invlpg(vma);
//
//         if (pma == 0 && flags == 0) { // unmap
//                 pm_decref(old_page);
//         }
//         return true;
// }
//
// bool vmm_map(virt_addr_t vma, phys_addr_t pma, int flags) {
//         return __vmm_map(vma, pma, flags | PAGE_PRESENT, false);
// }
//
// void vmm_map_range(virt_addr_t vma, phys_addr_t pma, size_t len, int flags) {
//         assert((vma & PAGE_OFFSET_4K) == 0);
//         assert((pma & PAGE_OFFSET_4K) == 0);
//         len = round_up(len, PAGE_SIZE);
//         for (size_t i=0; i<len; i+=PAGE_SIZE) {
//                 __vmm_map(vma + i, pma + i, flags | PAGE_PRESENT, false);
//         }
// }
//
// void vmm_create_unbacked(virt_addr_t vma, int flags) {
//         __vmm_map(vma, 0, flags | PAGE_UNBACKED, false);
// }
//
// void vmm_create_unbacked_range(virt_addr_t vma, size_t len, int flags) {
//         // printf("%p\n", vma);
//         assert((vma & PAGE_OFFSET_4K) == 0);
//         len = round_up(len, PAGE_SIZE);
//         for (size_t i=0; i<len; i+=PAGE_SIZE) {
//                 __vmm_map(vma + i, 0, flags | PAGE_UNBACKED, false);
//         }
// }
//
// bool vmm_unmap(virt_addr_t vma) {
//         return __vmm_map(vma, 0, 0, true);
// }
//
// void vmm_unmap_range(virt_addr_t vma, size_t len) {
//         assert((vma & PAGE_OFFSET_4K) == 0);
//         len = round_up(len, PAGE_SIZE);
//         for (size_t i=0; i<len; i+=PAGE_SIZE) {
//                 __vmm_map(vma + i, 0, 0, true);
//         }
// }
//
// void vmm_copy(virt_addr_t vma, phys_addr_t new_root, enum vmm_copy_op op) {
//         uintptr_t vm_root = running_process->vm_root;
//         uintptr_t *pte_ptr = vmm_pte_ptr(vma);
//         assert(pte_ptr);
//         uintptr_t pte = *pte_ptr;
//         phys_addr_t page = pte & PAGE_MASK_4K;
//         phys_addr_t new_page;
//         uintptr_t *new_ptr = __vmm_pte_ptr(vma, new_root, 4, 1);
//         assert(new_ptr);
//
//         if (is_unbacked(pte)) {
//                 *new_ptr = pte;
//                 return;
//         }
//
//         switch (op) {
//         case COPY_COW:
//                 *pte_ptr &= ~PAGE_WRITEABLE;
//                 *pte_ptr |= PAGE_COPYONWRITE;
//                 *new_ptr = *pte_ptr;
//                 invlpg(vma);
//                 pm_incref(page);
//                 break;
//         case COPY_SHARED:
//                 *new_ptr = pte;
//                 pm_incref(page);
//                 break;
//         case COPY_EAGER:
//                 new_page = pm_alloc();
//                 memcpy((void *)vma, (void *)(new_page + VMM_MAP_BASE), PAGE_SIZE);
//                 *new_ptr = (pte & PAGE_FLAGS_MASK) | new_page;
//                 break;
//         default:
//                 panic("illegal vm_copy operation");
//         }
// }
//
// void vmm_copy_region(virt_addr_t base, virt_addr_t top, phys_addr_t new_root, enum vmm_copy_op op) {
//         assert((base & PAGE_OFFSET_4K) == 0);
//         assert((top & PAGE_OFFSET_4K) == 0);
//
//         if (base == 0)  return;
//
//         for (size_t page=base; page<top; page+=PAGE_SIZE) {
//                 vmm_copy(page, new_root, op);
//         }
// }
//
// phys_addr_t vmm_fork(struct process *proc) {
//         disable_irqs();
//         phys_addr_t new_vm_root = pm_alloc();
//         uintptr_t *new_root_ptr = (uintptr_t *)(new_vm_root + VMM_MAP_BASE);
//
//         phys_addr_t vm_root = running_process->vm_root;
//         uintptr_t *vm_root_ptr = (uintptr_t *)(vm_root + VMM_MAP_BASE);
//
//         // copy the top half to the new table;
//         memcpy(new_root_ptr + 256, vm_root_ptr + 256, 256 * sizeof(uintptr_t));
//         memset(new_root_ptr, 0, 256 * sizeof(uintptr_t));
//
//         struct mm_region *regions = &running_process->mm_regions[0];
//         for (size_t i=0; i<NREGIONS; i++) {
//                 vmm_copy_region(regions[i].base, regions[i].top, new_vm_root, COPY_COW);
//         }
//         memcpy(&proc->mm_regions, &running_process->mm_regions, sizeof(struct mm_region) * NREGIONS);
//         // reset_tlb();
//         enable_irqs();
//         return new_vm_root;
// }
//
// void __vmm_destroy_tree(phys_addr_t root, int level) {
//         size_t top = 512;
//         if (level == 4) top = 256;
//         uintptr_t *root_ptr = (uintptr_t *)(root + VMM_MAP_BASE);
//
//         for (size_t i=0; i<top; i++) {
//                 if (root_ptr[i] && level > 1) {
//                         __vmm_destroy_tree(root_ptr[i] & PAGE_ADDR_MASK, level-1);
//                 }
//                 if (root_ptr[i]) {
//                         pm_free(root_ptr[i] & PAGE_ADDR_MASK);
//                 }
//                 root_ptr[i] = 0;
//         }
// }
//
// void vmm_destroy_tree(phys_addr_t root) {
//         __vmm_destroy_tree(root, 4);
//         pm_free(root);
// }
//
// extern uintptr_t boot_p4_mapping;
// extern uintptr_t boot_p3_mapping;
//
// void vmm_early_init(void) {
//         // unmap initial low p4 entry
//         boot_p4_mapping = 0;
//         *(uintptr_t *)((uintptr_t)&boot_p3_mapping + VMM_MAP_BASE) = 0;
//
//         // hhstack_guard_page = 0
//         // remap ro_begin to ro_end read-only
// }
//
// enum fault_result vmm_do_page_fault(virt_addr_t fault_addr, enum x86_fault reason) {
//         uintptr_t pte, phy, cur, flags, new_flags;
//         uintptr_t *pte_ptr = vmm_pte_ptr(fault_addr);
//
//         // printf("page fault %p %#02x\n", fault_addr, reason);
//
//         if (!pte_ptr)  return FAULT_CRASH;
//         pte = *pte_ptr;
//         if (pte == 0)  return FAULT_CRASH;
//
//         if (reason & F_RESERVED)  return FAULT_CRASH;
//         if (reason & F_RESERVED)  return FAULT_CRASH;
//
//         if (is_unbacked(pte)) {
//                 phy = pm_alloc();
//                 *pte_ptr &= PAGE_FLAGS_MASK;
//                 *pte_ptr |= phy | PAGE_PRESENT;
//                 return FAULT_CONTINUE;
//         }
//
//         if ((pte & PAGE_COPYONWRITE) && (reason & F_WRITE)) {
//                 phy = pm_alloc();
//                 cur = pte & PAGE_ADDR_MASK;
//                 flags = pte & PAGE_FLAGS_MASK;
//
//                 memcpy((void *)(phy + VMM_MAP_BASE), (void *)(cur + VMM_MAP_BASE), PAGE_SIZE);
//                 pm_decref(cur);
//
//                 new_flags = flags & ~(PAGE_COPYONWRITE | PAGE_ACCESSED | PAGE_DIRTY);
//                 *pte_ptr = phy | new_flags | PAGE_WRITEABLE;
//                 invlpg(fault_addr);
//                 return FAULT_CONTINUE;
//         }
//
//         if (pte & PAGE_STACK_GUARD) {
//                 printf("Warning! Page fault in page marked stack guard!\n");
//                 return FAULT_CRASH;
//         }
//
//         return FAULT_CRASH;
// }

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
//             root: physicalmemory::alloc_zero(),
//         }
//     }
//
//     pub fn fork() -> Self {
//         Self {
//             root: fork_this_address_space(),
//         }
//     }
//
//     pub fn pte_mut(&mut self, v: VirtualAddress) -> Option<&mut PageTableEntry> {
//         Self::pte_mut_recursive(self.root, v, 4, false)
//     }
//
//     pub fn pte(&self, v: VirtualAddress) -> Option<PageTableEntry> {
//         Self::pte_mut_recursive(self.root, v, 4, false).cloned()
//     }
//
//     pub fn resolve(&self, v: VirtualAddress) -> Option<PhysicalAddress> {
//         self.pte(v).page().map(|a| a | v.page_offset())
//     }
//
//     pub fn map(&mut self, v: VirtualAddress, p: PhysicalAddress, flags: usize) -> Result<(), PagingError> {
//         *self.pte_mut(v)? = p | flags
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
//     fn pte_mut_recursuve(root: PhysicalAddress, v: VirtualAddress, level: usize, create: bool) -> Result<&mut PageTableEntry, PagingError> {
//         let level_offset = Self::offset(v, level);
//         let entry = &mut root[offset];
//         if level == 1 {
//             return Ok(entry);
//         }
//         if !(root[offset] & PageTableEntry::PRESENT) {
//             if create {
//                 Self::make_next_table(entry, v.is_higher_half());
//             } else {
//                 return Err(Other("Page Not Present"));
//             }
//         }
//         Self::pte_mut_recursive(entry.page(), v, level-1, create)
//     }
//
//     fn make_next_table(p: &mut PageTableEntry, kernel: bool) {
//         let flags = if kernel {
//             PageTableEntry::TABLE_FLAGS
//         } else {
//             PageTableEntry::TABLE_FLAGS | PageTableEntry::USERMODE
//         }
//         *p = physicalmemory::alloc_zero() | flags;
//     }
//
//     fn pte_mut_create(&mut self, v: VirtualAddress) -> &mut PageTableEntry {
//         Self::pte_mut_recursive(self.root, v, 4, true).expect("pte_mut_recursive failed")
//     }
// }