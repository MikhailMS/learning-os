/// 64-bit physical memory address
///
/// On `x86_64` only lower 52 bits are used, the upper 12 must be zero-ed
/// This wrapper struct would ensure that this is the case
#[repr(transparent)] // To ensure memory layout is exactly same as for u64
pub struct PhysAddr(u64);

impl PhysAddr {

    /// Create new Physical Address
    ///
    /// Function panics when address contains data in bits 53 to 64
    #[inline]
    pub const fn new(addr: u64) -> Self {
        Self::try_new(addr).expect("address passed should not contain anything in bits 53 to 64")
    }

    /// Creates new Physical Address without performing any checks
    ///
    /// # Safety
    /// Caller must ensure that bits 53..64 do not contain any data (set to zero)
    #[inline]
    pub const unsafe fn unsafe_new(addr: u64) -> Self {
        Physical(addr)
    }

    /// Tries to create a canonical physical address
    /// 
    /// If address is already canonical, then return it,
    /// otherwise try to perform sign extension of bit 47 to make the address canonical and return it.
    ///
    /// Function succeeds if bits 48 to 64 are either a correct sign extension (i.e. copies of bit 47) or all null.
    /// Else () is returned 
    #[inline]
    pub fn try_new(addr: u64) -> Result<Self, ()> {
        let p_addr = Self::new_truncate(addr);

        if p_addr == addr {
            Ok(p_addr) // correct physical address
        } else {
            ()         // incorrect physical address
        }
    }

    /// Create new Physical Address & throwing bits 53..64
    #[inline]
    pub const fn new_truncate(addr: u64) -> Self {
        // This way we clear up 12 leftmost bits (set to zero)
        PhysAddr( addr % (1 << 52) )
    }
}

