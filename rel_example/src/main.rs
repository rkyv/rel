#![deny(unsafe_op_in_unsafe_fn)]

use ::core::mem::MaybeUninit;
use ::mischief::{GhostRef, Slot, StaticToken};
use ::rel_slab_allocator::*;
use ::rel_util::Align16;
use ::situ::ops::DerefMutRaw;

fn rel_box() {
    use rel_alloc::{EmplaceIn, RelBox};
    use rel_core::I32;

    let mut backing = Align16(MaybeUninit::<[u8; 128]>::zeroed());

    let size = StaticToken::acquire(|mut token| {
        let bytes = Slot::new(&mut backing.0).unsize();
        let alloc =
            SlabAllocator::<_>::try_new_in(bytes, GhostRef::leak(&mut token))
                .unwrap();

        let int = 42.emplace_in::<I32>(alloc);
        println!("{int}");
        let emplaced_int =
            int.emplace_in::<RelBox<_, RelSlabAllocator<_>>>(alloc);
        println!("{emplaced_int}");

        assert!(alloc.deposit(emplaced_int).is_none());

        alloc.shrink_to_fit()
    });

    let mut backing_2 = Align16::frame(size);
    backing_2.slot().zero();
    unsafe {
        ::core::ptr::copy_nonoverlapping(
            backing.0.as_mut_ptr().cast::<u8>(),
            backing_2.as_mut_ptr().cast::<u8>(),
            size,
        );
    }

    StaticToken::acquire(|token| {
        let bytes = backing_2.slot().as_bytes();
        let alloc = SlabAllocator::<_>::try_from_bytes(bytes, token).unwrap();

        let mut emplaced_int = unsafe {
            alloc.withdraw_unchecked::<RelBox<I32, RelSlabAllocator<StaticToken>>>().unwrap()
        };
        *RelBox::deref_mut_raw(emplaced_int.as_mut()) = I32::from(10);
        println!("{emplaced_int}");
    });

    println!("bytes: {:?}", unsafe { backing_2.assume_init() });
}

fn rel_vec() {
    use ::rel_alloc::{vec, EmplaceIn, RelVec};
    use ::rel_core::I32;

    let mut backing = Align16(MaybeUninit::<[u8; 512]>::zeroed());

    StaticToken::acquire(|token| {
        let bytes = Slot::new(&mut backing.0).unsize();
        let alloc = SlabAllocator::<_>::try_new_in(bytes, token).unwrap();

        let mut vec = vec::New(alloc)
            .emplace_in::<RelVec<I32, RelSlabAllocator<_>>>(alloc);

        for i in 0..10 {
            RelVec::push(vec.as_mut(), i * i);
        }

        println!("initial ints: {vec:?}");

        assert!(alloc.deposit(vec).is_none());
    });

    StaticToken::acquire(|token| {
        let bytes = Slot::new(&mut backing.0).unsize();
        let alloc = SlabAllocator::<_>::try_from_bytes(bytes, token).unwrap();

        let mut vec = unsafe {
            alloc.withdraw_unchecked::<RelVec<I32, RelSlabAllocator<StaticToken>>>().unwrap()
        };

        println!("ints before: {vec:?}");

        for i in 10..20 {
            RelVec::push(vec.as_mut(), i * i);
        }

        println!("ints after: {vec:?}");
    });
}

fn rel_vec_rel_box() {
    use ::rel_alloc::{vec, EmplaceIn, RelBox, RelVec};
    use ::rel_core::I32;

    let mut backing = Align16(MaybeUninit::<[u8; 1024]>::zeroed());

    StaticToken::acquire(|token| {
        let bytes = Slot::new(&mut backing.0).unsize();
        let alloc = SlabAllocator::<_>::try_new_in(bytes, token).unwrap();

        let mut vec = vec::New(alloc).emplace_in::<RelVec<
            RelBox<I32, RelSlabAllocator<StaticToken>>,
            RelSlabAllocator<StaticToken>,
        >>(alloc);

        for i in 0..10 {
            let int = (i * i).emplace_in::<I32>(alloc);
            RelVec::push(vec.as_mut(), int);
        }

        println!("initial ints: {vec:?}");

        assert!(alloc.deposit(vec).is_none());
    });

    StaticToken::acquire(|token| {
        let bytes = Slot::new(&mut backing.0).unsize();
        let alloc = SlabAllocator::<_>::try_from_bytes(bytes, token).unwrap();

        let mut vec = unsafe {
            alloc
                .withdraw_unchecked::<RelVec<
                    RelBox<I32, RelSlabAllocator<StaticToken>>,
                    RelSlabAllocator<StaticToken>,
                >>()
                .unwrap()
        };

        println!("ints before: {vec:?}");

        for i in 10..20 {
            let int = (i * i).emplace_in::<I32>(alloc);
            RelVec::push(vec.as_mut(), int);
        }
        println!("ints after: {vec:?}");
    });
}

fn rel_string() {
    use ::rel_alloc::{string, EmplaceIn, RelString};

    let mut backing = Align16(MaybeUninit::<[u8; 1024]>::zeroed());

    StaticToken::acquire(|token| {
        let bytes = Slot::new(&mut backing.0).unsize();
        let alloc = SlabAllocator::<_>::try_new_in(bytes, token).unwrap();

        let s = string::Clone(alloc, "Hello world!")
            .emplace_in::<RelString<RelSlabAllocator<_>>>(alloc);

        println!("string: '{s}'");
    });
}

fn main() {
    rel_box();
    rel_vec();
    rel_vec_rel_box();
    rel_string();
}
