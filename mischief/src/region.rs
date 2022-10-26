use ::heresy::alloc::Allocator;

/// A contiguous memory region.
///
/// Each type that implements `Region` corresponds to a single allocated object
/// at runtime. That `Region` type is said to "contain" some memory segment if
/// the segment is completely contained within the bounds of the allocated
/// object. While each region must correspond to a single allocated object, each
/// allocated object may correspond with multiple `Region` types.
///
/// If memory segments are contained in the same region type, then computing the
/// the offset between them is guaranteed to be safe. However, because different
/// `Region` types may correspond with the same allocated object, computing the
/// offset betwewen memory segments in different regions may or may not be safe.
///
/// Note that while a type implementing `Region` must correspond with a single
/// allocated object, it does not need to know anything about that allocated
/// object.
///
/// # Safety
///
/// This type must correspond to a single unique allocated object.
pub unsafe trait Region {}

/// An `Allocator` that allocates inside a single contiguous memory region.
///
/// # Safety
///
/// The pointers returned from a `RegionalAllocator`'s `Allocator`
/// implementation must always be contained in its associated `Region`.
pub unsafe trait RegionalAllocator: Allocator {
    /// The region type for this allocator.
    type Region: Region;
}
