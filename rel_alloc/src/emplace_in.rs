use ::mischief::{Frame, In, Metadata, RegionalAllocator};
use ::ptr_meta::Pointee;
use ::rel_core::Emplace;
use ::situ::{DropRaw, OwnedVal};

/// An extension trait for `Emplace` that provides an allocating emplacement
/// function.
pub trait EmplaceIn<A: RegionalAllocator> {
    /// Emplaces a value into a new `OwnedVal` allocated from the given
    /// allocator and returns it.
    #[must_use]
    fn emplace_in<T>(self, alloc: A) -> OwnedVal<T, A>
    where
        T: DropRaw + Pointee + ?Sized,
        <T as Pointee>::Metadata: Metadata<T>,
        Self: Emplace<T, A::Region>;
}

impl<E, A> EmplaceIn<A> for E
where
    A: RegionalAllocator,
{
    #[must_use]
    fn emplace_in<T>(self, alloc: A) -> OwnedVal<T, A>
    where
        T: DropRaw + Pointee + ?Sized,
        <T as Pointee>::Metadata: Metadata<T>,
        Self: Emplace<T, A::Region>,
    {
        let frame =
            // SAFETY: The pointer metadata is from `emplaced_meta`, which is
            // guaranteed to be valid for a pointer to `T`.
            unsafe { Frame::new_unsized_in(self.emplaced_meta(), alloc) };

        let mut frame = In::new(frame);
        let slot = frame.slot();

        // SAFETY: We just allocated the slot in a frame with the metadata from
        // `emplaced_meta`.
        unsafe {
            self.emplace_unsized_unchecked(slot);
        }
        // SAFETY: `emplace` is guaranteed to initialize the slot. That slot is
        // from the frame, so the frame is initialized.
        unsafe { OwnedVal::assume_init(In::into_inner(frame)) }
    }
}
