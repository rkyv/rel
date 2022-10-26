use ::mischief::Region;

/// A type with values that are always located in a particular region.
///
/// While values of types that implement `Pinned<R>` must be located in `R`,
/// uninitialized values do not. This is why [`Slot`](::mischief::Slot) cannot
/// implement `Within<R>` when its type impelements `Pinned<R>`.
///
/// # Safety
///
/// Values of this type must only ever be located in `R`.
pub unsafe trait Pinned<R: Region> {}
