use ::core::mem;
use ::rand::Rng;

pub trait Generate {
    fn generate<R: Rng>(rng: &mut R) -> Self;
}

impl Generate for () {
    fn generate<R: Rng>(_: &mut R) -> Self {}
}

impl Generate for bool {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        rng.gen_bool(0.5)
    }
}

macro_rules! impl_generate {
    ($ty:ty) => {
        impl Generate for $ty {
            fn generate<R: Rng>(rng: &mut R) -> Self {
                rng.gen()
            }
        }
    };
}

impl_generate!(u8);
impl_generate!(u16);
impl_generate!(u32);
impl_generate!(u64);
impl_generate!(u128);
impl_generate!(usize);
impl_generate!(i8);
impl_generate!(i16);
impl_generate!(i32);
impl_generate!(i64);
impl_generate!(i128);
impl_generate!(isize);
impl_generate!(f32);
impl_generate!(f64);

macro_rules! impl_tuple {
    () => {};
    ($first:ident, $($rest:ident,)*) => {
        impl<$first: Generate, $($rest: Generate,)*> Generate for ($first, $($rest,)*) {
            fn generate<R: Rng>(rng: &mut R) -> Self {
                ($first::generate(rng), $($rest::generate(rng),)*)
            }
        }

        impl_tuple!($($rest,)*);
    };
}

impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11,);

macro_rules! impl_array {
    () => {};
    ($len:literal, $($rest:literal,)*) => {
        impl<T: Generate> Generate for [T; $len] {
            fn generate<R: Rng>(rng: &mut R) -> Self {
                let mut result = mem::MaybeUninit::<Self>::uninit();
                let result_ptr = result.as_mut_ptr().cast::<T>();
                for i in 0..$len {
                    unsafe {
                        result_ptr.add(i).write(Generate::generate(rng));
                    }
                }
                unsafe {
                    result.assume_init()
                }
            }
        }

        impl_array!($($rest,)*);
    }
}

impl_array!(
    31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13,
    12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
);

impl<T: Generate> Generate for Option<T> {
    fn generate<R: Rng>(rng: &mut R) -> Self {
        if rng.gen_bool(0.5) {
            Some(Generate::generate(rng))
        } else {
            None
        }
    }
}

pub fn generate_vec<R: Rng, T: Generate>(rng: &mut R, len: usize) -> Vec<T> {
    let mut result = Vec::with_capacity(len);
    for _ in 0..len {
        result.push(Generate::generate(rng));
    }
    result
}

pub fn default_rng() -> impl Rng {
    use ::rand_pcg::Lcg64Xsh32;

    // Nothing up our sleeves; state and stream are first 20 digits of pi.
    const STATE: u64 = 3141592653;
    const STREAM: u64 = 5897932384;

    Lcg64Xsh32::new(STATE, STREAM)
}
