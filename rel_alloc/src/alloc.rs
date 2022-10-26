//! Memory allocation APIs.

use ::mischief::RegionalAllocator;
use ::rel_core::Emplace;
use ::situ::{alloc::RawRegionalAllocator, DropRaw};

/// An `Allocator` that is suitable for allocating relative types.
///
/// # Safety
///
/// When emplaced as an `E`, the emplaced `E` must function analogously to the
/// original allocator. Specifically, it must return the same results when
/// calling the analogous allocator methods from `RawAllocator` and share the
/// same state between the two (e.g. allocating with one and freeing with the
/// other must be safe and function properly).
pub unsafe trait RelAllocator<E>:
    RegionalAllocator + Emplace<E, Self::Region> + Sized
where
    E: DropRaw + RawRegionalAllocator<Region = Self::Region>,
{
}
