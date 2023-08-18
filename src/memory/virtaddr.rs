use bit_field::BitField;

/// 64-bit virtual memory address
///
/// On `x86_64` only lower 48 bits are used, the upper 16 must be exact copy 
/// of 47's bit
/// Addresses that comply with such are called `canonical`
///
/// This wrapper struct would ensure that virtual addresses are always canonicall
#[repr(transparent)] // To ensure memory layout is exactly same as for u64
pub struct VirtAddr(u64);

impl VirtAddr {

    /// Create new canonical Virtual Address
    ///
    /// Function panics when address contains data in bits 48 to 64
    #[inline]
    pub fn new(addr: u64) -> Self {
        Self::try_new(addr).expect("address passed should not contain anything in bits 48 to 64")
    }

    /// Tries to create a canonical virtual address
    /// 
    /// If address is already canonical, then return it,
    /// otherwise try to perform sign extension of bit 47 to make the address canonical and return it.
    ///
    /// Function succeeds if bits 48 to 64 are either a correct sign extension (i.e. copies of bit 47) or all null.
    /// Else () is returned 
    #[inline]
    pub fn try_new(addr: u64) -> Result<Self, ()> {
        match addr.get_bits(47..64) {
            // TODO: Figure out why we have 0x1ffff here
            0 | 0x1ffff => VirtAddr(addr),               // canonical address
            1           => VirtAddr::new_truncate(addr), // not canonical - requires sign extension
            _           => ()                            // incorrect address
        }
    }

    /// Create new canonical Virtual Address & throwing bits 48..64
    ///
    /// Function performs sign extension of bit 47 to ensure VirtAddr is canonical
    #[inline]
    pub const fn new_truncate(addr: u64) -> Self {
        // 1. Left shift 16 bits so that 47th bit becomes a sign bit
        // 2. Convert u64 to i64 and Right shift 16 bits to perform sign extension
        // 3. Convert i64 back to u64
        VirtAddr( ((addr << 16) as i64 >> 16) as u64 )
    }

    /// Creates new Virtual Address without performing any checks
    ///
    /// # Safety
    /// Caller must ensure that bits 48..64 are equal to bit 47
    #[inline]
    pub const unsafe fn unsafe_new(addr: u64) -> Self {
        VirtAddr(addr)
    }

    /// Returns 12-bits page offset (lowest 12 bits of the address) of Virtual Address
    #[inline]
    pub const fn page_offset(self) -> PageOffset {
        PageOffset::new_truncate(self.0 as u16)
    }

    /// Returns 9-bits level 1 page table index (first group of 9 bits after page offset) of Virtual Address
    #[inline]
    pub const fn p1_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate(self.0 >> 12 as u16)
    }

    /// Returns 9-bits level 2 page table index (second group of 9 bits after page offset) of Virtual Address
    #[inline]
    pub const fn p2_index(self) -> PageOffset {
        PageTableIndex::new_truncate(self.0 >> 12 >> 9 as u16)
    }

    /// Returns 9-bits level 3 page table index (third group of 9 bits after page offset) of Virtual Address
    #[inline]
    pub const fn p3_index(self) -> PageOffset {
        PageTableIndex::new_truncate(self.0 >> 12 >> 9 >> 9 as u16)
    }

    /// Returns 9-bits level 4 page table index (fourth group of 9 bits after page offset) of Virtual Address
    #[inline]
    pub const fn p4_index(self) -> PageOffset {
        PageTableIndex::new_truncate(self.0 >> 12 >> 9 >> 9 >> 9 as u16)
    }

    /// Returns 9-bits of provided level page table index of Virtual Address
    #[inline]
    pub const fn page_table_index(self, level: PageTableLevel) -> PageOffset {
        PageTableIndex::new_truncate(self.0 >> 12 >> ((level as u8 - 1) * 9)) as u16)
    }
}
