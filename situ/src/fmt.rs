//! Utilities for formatting and printing raw values.

use ::core::fmt::{Debug, Display, Error, Formatter};

use crate::Ref;

/// `?` formatting for raw references.
pub trait DebugRaw {
    /// Formats the value using the given formatter.
    fn fmt_raw(this: Ref<'_, Self>, f: &mut Formatter<'_>)
        -> Result<(), Error>;
}

impl<T: Debug + ?Sized> DebugRaw for T {
    fn fmt_raw(
        this: Ref<'_, Self>,
        f: &mut Formatter<'_>,
    ) -> Result<(), Error> {
        Debug::fmt(&*this, f)
    }
}

/// Format trait for an empty format, `{}`.
pub trait DisplayRaw {
    /// Formats the value using the given formatter.
    fn fmt_raw(this: Ref<'_, Self>, f: &mut Formatter<'_>)
        -> Result<(), Error>;
}

impl<T: Display + ?Sized> DisplayRaw for T {
    fn fmt_raw(
        this: Ref<'_, Self>,
        f: &mut Formatter<'_>,
    ) -> Result<(), Error> {
        Display::fmt(&*this, f)
    }
}
