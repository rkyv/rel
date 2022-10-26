use ::syn::{parse, token::Eq, LitStr};

/// The value of an attribute like `#[key = "value"]`.
pub struct AttrValue<T> {
    /// The `=` token between the key and value.
    pub eq_token: Eq,
    /// The value parsed from a string literal.
    pub value: T,
}

impl<T: parse::Parse> parse::Parse for AttrValue<T> {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        Ok(AttrValue {
            eq_token: input.parse::<Eq>()?,
            value: input.parse::<LitStr>()?.parse::<T>()?,
        })
    }
}
