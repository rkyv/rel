use ::core::marker::PhantomData;

use crate::Unique;

/// A type of which there can only ever exist one value.
#[repr(transparent)]
pub struct StaticToken<'id>(PhantomData<fn(&'id ()) -> &'id ()>);

impl<'id> StaticToken<'id> {
    /// Calls the given function with a fresh, unique token that will never be
    /// used again.
    #[inline]
    pub fn acquire<R, F>(f: F) -> R
    where
        F: for<'new_id> FnOnce(StaticToken<'new_id>) -> R,
    {
        f(StaticToken(PhantomData))
    }
}

// SAFETY: All `StaticToken`s have fresh, unused lifetimes and so are unique
// types.
unsafe impl<'id> Unique for StaticToken<'id> {}

/// The runtime token is simultaneously acquired elsewhere.
#[derive(Debug)]
pub struct RuntimeTokenError;

/// Creates a token with a fresh type that is checked for uniqueness at runtime.
#[macro_export]
macro_rules! runtime_token {
    ($name:ident) => {
        $crate::runtime_token!(@impl $name);
    };
    (pub $name:ident) => {
        $crate::runtime_token!(@impl $name pub);
    };
    (pub ($($vis:tt)*) $name:ident) => {
        $crate::runtime_token!(@impl $name pub($($vis)*));
    };
    (@impl $name:ident $($vis:tt)*) => {
        #[repr(transparent)]
        $($vis)* struct $name(::core::marker::PhantomData<()>);

        const _: () = {
            static ALIVE: ::core::sync::atomic::AtomicBool =
                ::core::sync::atomic::AtomicBool::new(false);

            impl Drop for $name {
                #[inline]
                fn drop(&mut self) {
                    ALIVE.compare_exchange(
                        true,
                        false,
                        ::core::sync::atomic::Ordering::AcqRel,
                        ::core::sync::atomic::Ordering::Acquire,
                    ).unwrap();
                }
            }

            impl $name {
                /// Acquires the token.
                ///
                /// # Panics
                ///
                /// Panics if the token is still acquired elsewhere.
                #[inline]
                pub fn acquire() -> Self {
                    Self::try_acquire().unwrap()
                }

                /// Attempts to acquire the token.
                ///
                /// Returns an error if the token is still acquired elsewhere.
                #[inline]
                pub fn try_acquire() ->
                    ::core::result::Result<Self, $crate::RuntimeTokenError>
                {
                    let result = ALIVE.compare_exchange(
                        false,
                        true,
                        ::core::sync::atomic::Ordering::AcqRel,
                        ::core::sync::atomic::Ordering::Acquire,
                    );

                    match result {
                        Ok(_) => ::core::result::Result::Ok(
                            $name(::core::marker::PhantomData),
                        ),
                        Err(_) => ::core::result::Result::Err(
                            $crate::RuntimeTokenError,
                        ),
                    }
                }
            }

            // SAFETY: `$name` can only be constructed by flipping `ALIVE` from
            // `false` to `true`, which can only happen one at a time.
            // Therefore, only one `$name` can exist at a time. The token will
            // flip it back to `false` when it is dropped, which destroys the
            // unique value.
            unsafe impl $crate::Unique for $name {}
        };
    };
}

#[cfg(test)]
mod tests {
    use crate::Unique;

    #[inline]
    fn assert_unique<T: Unique>() {}

    #[test]
    fn static_token() {
        use crate::StaticToken;

        StaticToken::acquire(|_: StaticToken| {
            assert_unique::<StaticToken>();
        });
    }

    #[test]
    fn runtime_token() {
        runtime_token!(Foo);
        assert_unique::<Foo>();

        let foo: Foo = Foo::acquire();
        assert!(matches!(Foo::try_acquire(), Err(_)));
        drop(foo);
        assert!(matches!(Foo::try_acquire(), Ok(_)));
    }

    #[test]
    #[should_panic]
    fn runtime_token_duplicate() {
        runtime_token!(Foo);
        assert_unique::<Foo>();

        let foo: Foo = Foo::acquire();
        let bar: Foo = Foo::acquire();
        drop(foo);
        drop(bar);
    }
}
