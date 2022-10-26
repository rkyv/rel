use ::core::ops::{Deref, DerefMut};

use crate::{GhostRef, Singleton, Slot, Unique};

/// A type that can lease access to a type without any context.
pub trait Static {
    /// The unique type that can be used to lease this static memory location.
    type Unique: Unique;
    /// The type that a reference can be created to.
    type Target: 'static;

    /// Returns a mutable reference to a slot of the target type.
    ///
    /// # Safety
    ///
    /// The caller must hold a mutable borrow of the `Unique`.
    unsafe fn slot() -> Slot<'static, Self::Target>;
}

/// A lease on a static memory location for a statically-checked lifetime.
#[repr(transparent)]
pub struct Lease<'scope, S: Static>(GhostRef<&'scope mut S::Unique>);

// SAFETY: `Lease<'scope, S>` contains a `GhostRef<&'scope mut S::Unique>`, so
// if that field is `Unique` then the `Lease` is also `Unique`.
unsafe impl<'scope, S: Static> Unique for Lease<'scope, S> where
    GhostRef<&'scope mut S::Unique>: Unique
{
}

impl<'scope, S: Static> Drop for Lease<'scope, S> {
    fn drop(&mut self) {
        // SAFETY:
        // - We hold a mutable borrow of `S::Unique`.
        // - Because the `Lease` is being dropped, there are no other references
        //   to the value in the static variable. Therefore, we may consume the
        //   value by dropping it.
        unsafe { S::slot().assume_init_drop() }
    }
}

impl<'scope, S: Static> Lease<'scope, S> {
    /// Creates a new scope from a unique borrow and an initial value.
    pub fn new(x: &'scope mut S::Unique, value: S::Target) -> Self {
        // SAFETY:
        // - We hold a mutable borrow of `S::Unique`.
        // - Because we are the only holders of the mutable borrow, we may treat
        //   the slot as owned.
        let mut slot = unsafe { S::slot() };
        slot.write(value);
        Self(GhostRef::leak(x))
    }

    /// Creates a shared borrow of this scoped static.
    pub fn borrow(&self) -> StaticRef<&Self> {
        StaticRef::leak(self)
    }

    /// Creates a mutable borrow of this scoped static.
    pub fn borrow_mut(&mut self) -> StaticRef<&mut Self> {
        StaticRef::leak(self)
    }
}

/// A reference to some static value.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct StaticRef<T>(GhostRef<T>);

// SAFETY: `StaticRef<T>` contains a `GhostRef<&'scope mut S::Unique>`, so if
// that field is `Unique` then the `StaticRef` is also `Singleton`.
unsafe impl<T> Unique for StaticRef<T> where GhostRef<T>: Unique {}

// SAFETY: `StaticRef<T>` contains only a `GhostRef<T>`, so if that field is
// `Singleton` then the `StaticRef` is also `Singleton`.
unsafe impl<T> Singleton for StaticRef<T> where GhostRef<T>: Singleton {}

impl<T> StaticRef<T> {
    /// Creates a new `StaticRef`.
    pub fn leak(x: T) -> Self {
        StaticRef(GhostRef::leak(x))
    }
}

impl<'borrow, 'scope, S> Deref for StaticRef<&'borrow Lease<'scope, S>>
where
    'scope: 'borrow,
    S: Static,
{
    type Target = S::Target;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // - `StaticRef` transitively holds a mutable borrow of `S::Unique`.
        // - The borrowed `Lease` initialized the slot when it was created.
        // - Because the borrow of `Lease` is shared, we treat the slot as
        //   shared.
        unsafe { S::slot().assume_init_ref() }
    }
}

impl<'borrow, 'scope, S> Deref for StaticRef<&'borrow mut Lease<'scope, S>>
where
    'scope: 'borrow,
    S: Static,
{
    type Target = S::Target;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // - `StaticRef` transitively holds a mutable borrow of `S::Unique`.
        // - The borrowed `Lease` initialized the slot when it was created.
        // - Because the borrow of `Lease` is unique, we may treat the slot as
        //   shared or mutable.
        unsafe { S::slot().assume_init_ref() }
    }
}

impl<'borrow, 'scope, S> DerefMut for StaticRef<&'borrow mut Lease<'scope, S>>
where
    'scope: 'borrow,
    S: Static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:
        // - `StaticRef` transitively holds a mutable borrow of `S::Unique`.
        // - The borrowed `Lease` initialized the slot when it was created.
        // - Because the borrow of `Lease` is unique, we may treat the slot as
        //   shared or mutable.
        unsafe { S::slot().assume_init_mut() }
    }
}

/// Creates a type that provides safe access to a static variable using a unique
/// value.
#[macro_export]
macro_rules! lease_static {
    ($unique:ty => $name:ident: $ty:ty) => {
        $crate::lease_static!(@declare $name);
        $crate::lease_static!(@impl $unique => $name: $ty)
    };
    ($unique:ty => pub $name:ident: $ty:ty) => {
        $crate::lease_static!(@declare $name pub);
        $crate::lease_static!(@impl $unique => $name: $ty)
    };
    ($unique:ty => pub ($($vis:tt)*) $name:ident: $ty:ty) => {
        $crate::lease_static!(@declare $name pub($($vis)*));
        $crate::lease_static!(@impl $unique => $name: $ty)
    };
    (@declare $name:ident $($vis:tt)*) => {
        $($vis)* struct $name(::core::marker::PhantomData<()>);
    };
    (@impl $unique:ty => $name:ident: $target:ty) => {
        const _: () = {
            use ::core::mem::MaybeUninit;
            static mut VALUE: MaybeUninit<$target> = MaybeUninit::uninit();

            impl $crate::Static for $name {
                type Unique = $unique;
                type Target = $target;

                unsafe fn slot() -> $crate::Slot<'static, Self::Target> {
                    // SAFETY: Only one `Lease` can have access to the slot at a
                    // time.
                    unsafe { $crate::Slot::new(&mut VALUE) }
                }
            }
        };
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn vend() {
        use crate::{runtime_token, Lease};

        struct Gumball {
            size: i32,
        }

        runtime_token!(Quarter);
        lease_static!(Quarter => Vend: Gumball);

        let mut quarter = Quarter::acquire();
        let mut vend = Lease::<Vend>::new(&mut quarter, Gumball { size: 100 });

        assert_eq!(::core::mem::size_of_val(&vend.borrow()), 0);

        let mut gumball = vend.borrow_mut();
        gumball.size = 6;

        assert_eq!(vend.borrow().size, 6);

        let mut gumball_2 = vend.borrow_mut();
        gumball_2.size = 4;

        assert_eq!(vend.borrow().size, 4);
    }
}
