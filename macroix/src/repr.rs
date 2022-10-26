//! `repr` attribute AST nodes.

use ::core::fmt;
use ::proc_macro2::{Span, TokenStream};
use ::quote::{ToTokens, TokenStreamExt};
use ::syn::{
    parenthesized,
    parse,
    parse2,
    token::{Comma, Paren},
    Error,
    Ident,
    LitInt,
};

/// An integer `repr`.
pub enum Int {
    /// `repr(i8)`
    I8,
    /// `repr(i16)`
    I16,
    /// `repr(i32)`
    I32,
    /// `repr(i64)`
    I64,
    /// `repr(i128)`
    I128,
    /// `repr(isize)`
    Isize,
    /// `repr(u8)`
    U8,
    /// `repr(u16)`
    U16,
    /// `repr(u32)`
    U32,
    /// `repr(u64)`
    U64,
    /// `repr(u128)`
    U128,
    /// `repr(usize)`
    Usize,
}

impl Int {
    /// Returns a `&'static str` representing the integer type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Int::I8 => "i8",
            Int::I16 => "i16",
            Int::I32 => "i32",
            Int::I64 => "i64",
            Int::I128 => "i128",
            Int::Isize => "isize",
            Int::U8 => "u8",
            Int::U16 => "u16",
            Int::U32 => "u32",
            Int::U64 => "u64",
            Int::U128 => "u128",
            Int::Usize => "usize",
        }
    }
}

impl ToTokens for Int {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(Ident::new(self.as_str(), Span::call_site()));
    }
}

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The base kind of a custom `repr`.
pub enum BaseKind {
    /// `repr(C)`
    C,
    /// `repr(transparent)`
    Transparent,
    /// `repr(i*)`/`repr(u*)`
    Primitive(Int),
}

impl BaseKind {
    /// Returns a `&'static str` representing the base kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            BaseKind::C => "C",
            BaseKind::Transparent => "transparent",
            BaseKind::Primitive(int) => int.as_str(),
        }
    }
}

impl ToTokens for BaseKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(Ident::new(self.as_str(), Span::call_site()));
    }
}

impl fmt::Display for BaseKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A base `repr` AST node.
pub struct Base {
    /// The kind of the base `repr`.
    pub kind: BaseKind,
    /// The identifier corresponding to the base `repr`.
    pub kind_token: Ident,
}

/// A primitive type modifier for `repr(C)` enums.
pub struct PrimitiveType {
    /// The integer primitive type.
    pub int: Int,
    /// The identifier corresponding to the primitive type modifer.
    pub int_token: Ident,
}

/// A `repr` modifier argument like the explicit alignment for `align` and
/// `packed`.
pub struct ModifierArg {
    /// The enclosing parentheses.
    pub paren_token: Paren,
    /// The argument value.
    pub value: LitInt,
}

impl parse::Parse for ModifierArg {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let value;
        let paren_token = parenthesized!(value in input);
        Ok(Self {
            paren_token,
            value: value.parse::<LitInt>()?,
        })
    }
}

/// The kind of a custom `repr` modifier.
pub enum ModifierKind {
    /// `repr(align(N))`
    Align(ModifierArg),
    /// `repr(packed)` or `repr(packed(N))`
    Packed(Option<ModifierArg>),
}

impl fmt::Display for ModifierKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Align(ModifierArg { value, .. }) => {
                write!(f, "align({value})")
            }
            Self::Packed(None) => write!(f, "packed"),
            Self::Packed(Some(ModifierArg { value, .. })) => {
                write!(f, "packed({value})")
            }
        }
    }
}

/// A `repr` modifier AST node.
pub struct Modifier {
    /// The kind of the `repr` modifier.
    pub kind: ModifierKind,
    /// The identifier corresponding to the `repr` modifier.
    pub kind_token: Ident,
}

/// A complete `repr` AST node.
pub struct Repr {
    /// The parentheses surrounding the arguments to `repr`.
    pub paren_token: Paren,
    /// The base `repr`, if any.
    pub base: Option<Base>,
    /// `repr(C, i*)`/`repr(C, u*)`
    pub primitive_type: Option<PrimitiveType>,
    /// The `repr` modifiers, if any.
    pub modifier: Option<Modifier>,
}

impl Repr {
    /// Merges the tokens of an attribute argument into an optional `Repr`.
    pub fn merge_attr(
        this: &mut Option<Self>,
        tokens: TokenStream,
    ) -> Result<(), Error> {
        let other = parse2(tokens)?;
        if let Some(this) = this.as_mut() {
            this.merge(other)?;
        } else {
            *this = Some(other);
        }
        Ok(())
    }

    /// Merges two parsed `repr`s together, returning an error if they are
    /// incompatible.
    pub fn merge(&mut self, other: Self) -> Result<(), Error> {
        if let Some(self_base) = &self.base {
            if let Some(other_base) = other.base {
                return Err(Error::new_spanned(
                    other_base.kind_token,
                    format!("base repr `{}` already specified", self_base.kind),
                ));
            }
        } else {
            self.base = other.base;
        }

        if let Some(self_primitive_type) = &self.primitive_type {
            if let Some(other_primitive_type) = other.primitive_type {
                return Err(Error::new_spanned(
                    other_primitive_type.int_token,
                    format!(
                        "primitive type `{}` already specified",
                        self_primitive_type.int,
                    ),
                ));
            }
        } else {
            self.primitive_type = other.primitive_type;
        }

        if let Some(self_modifier) = &self.modifier {
            if let Some(other_modifier) = other.modifier {
                return Err(Error::new_spanned(
                    other_modifier.kind_token,
                    format!(
                        "repr modifier `{}` already specified",
                        self_modifier.kind,
                    ),
                ));
            }
        } else {
            self.modifier = other.modifier;
        }

        Ok(())
    }
}

impl parse::Parse for Repr {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let mut base = None;
        let mut primitive_type = None;
        let mut try_set_base = |kind, kind_token| {
            let add_base = Base { kind, kind_token };

            match (base.take(), add_base) {
                (None, add_base) => base = Some(add_base),
                // Combining C + primitive or primitive + C
                (
                    Some(Base {
                        kind: BaseKind::C,
                        kind_token,
                    }),
                    Base {
                        kind: BaseKind::Primitive(int),
                        kind_token: int_token,
                    },
                )
                | (
                    Some(Base {
                        kind: BaseKind::Primitive(int),
                        kind_token: int_token,
                    }),
                    Base {
                        kind: BaseKind::C,
                        kind_token,
                    },
                ) => {
                    base = Some(Base {
                        kind: BaseKind::C,
                        kind_token,
                    });
                    primitive_type = Some(PrimitiveType { int, int_token });
                }
                (Some(base), add_base) => {
                    return Err(Error::new_spanned(
                        add_base.kind_token,
                        format!("base repr `{}` already specified", base.kind),
                    ));
                }
            }

            Ok(())
        };
        let mut modifier = None;
        let mut try_set_modifier = |kind, kind_token| {
            modifier.replace(Modifier { kind, kind_token }).map_or(
                Ok(()),
                |old| {
                    Err(Error::new_spanned(
                        old.kind_token,
                        "repr modifier already specified",
                    ))
                },
            )
        };

        let args;
        let paren_token = parenthesized!(args in input);
        while !args.is_empty() {
            let token = args.parse::<Ident>()?;
            if token == "C" {
                try_set_base(BaseKind::C, token)?;
            } else if token == "transparent" {
                try_set_base(BaseKind::Transparent, token)?;
            } else if token == "i8" {
                try_set_base(BaseKind::Primitive(Int::I8), token)?;
            } else if token == "i16" {
                try_set_base(BaseKind::Primitive(Int::I16), token)?;
            } else if token == "i32" {
                try_set_base(BaseKind::Primitive(Int::I32), token)?;
            } else if token == "i64" {
                try_set_base(BaseKind::Primitive(Int::I64), token)?;
            } else if token == "i128" {
                try_set_base(BaseKind::Primitive(Int::I128), token)?;
            } else if token == "isize" {
                try_set_base(BaseKind::Primitive(Int::Isize), token)?;
            } else if token == "u8" {
                try_set_base(BaseKind::Primitive(Int::U8), token)?;
            } else if token == "u16" {
                try_set_base(BaseKind::Primitive(Int::U16), token)?;
            } else if token == "u32" {
                try_set_base(BaseKind::Primitive(Int::U32), token)?;
            } else if token == "u64" {
                try_set_base(BaseKind::Primitive(Int::U64), token)?;
            } else if token == "u128" {
                try_set_base(BaseKind::Primitive(Int::U128), token)?;
            } else if token == "usize" {
                try_set_base(BaseKind::Primitive(Int::Usize), token)?;
            } else if token == "align" {
                try_set_modifier(
                    ModifierKind::Align(args.parse::<ModifierArg>()?),
                    token,
                )?;
            } else if token == "packed" {
                try_set_modifier(
                    ModifierKind::Packed(args.parse::<ModifierArg>().ok()),
                    token,
                )?;
            } else {
                return Err(Error::new_spanned(token, "invalid repr argument"));
            }
            if args.peek(Comma) {
                args.parse::<Comma>()?;
            }
        }

        Ok(Repr {
            paren_token,
            base,
            primitive_type,
            modifier,
        })
    }
}
