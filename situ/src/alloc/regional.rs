use ::mischief::Region;

use crate::alloc::RawAllocator;

/// A `RawAllocator` that allocates inside a single contiguous memory region.
///
/// # Safety
///
/// The pointers returned from a `RawRegionalAllocator`'s `RawAllocator`
/// implementation must always be contained in its associated `Region`.
pub unsafe trait RawRegionalAllocator: RawAllocator {
    /// The region type for this allocator.
    type Region: Region;
}
