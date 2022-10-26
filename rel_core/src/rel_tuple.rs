//! Relative versions of tuples.

use ::mischief::{In, Region, Slot};
use ::munge::munge;
use ::ptr_meta::Pointee;
use ::situ::DropRaw;

use crate::{Emplace, EmplaceExt, Move, Portable};

macro_rules! define_tuple {
    (
        $n:expr,
        $ident:ident<$($types:ident),*>,
        ($($emplace_types:ident),*),
        $($indices:tt,)*
    ) => {
        #[doc = concat!("A relative ", stringify!($n), "-tuple")]
        #[derive(DropRaw, Move, Portable)]
        #[rel_core = "crate"]
        #[repr(C)]
        pub struct $ident<$($types),*>($($types),*);

        // SAFETY:
        // - `emplaced_meta` returns `()`, the only valid metadata for `Sized`
        //   types.
        // - `emplace_unsized_unchecked` initializes its `out` parameter by
        //   emplacing to each of its fields.
        unsafe impl<$($types,)* $($emplace_types,)* R: Region>
            Emplace<$ident<$($types),*>, R> for ($($emplace_types,)*)
        where
            $(
                $types: DropRaw,
                $emplace_types: Emplace<$types, R>,
            )*
        {
            fn emplaced_meta(
                &self,
            ) -> <$ident<$($types),*> as Pointee>::Metadata {}

            #[allow(non_snake_case)]
            unsafe fn emplace_unsized_unchecked(
                self,
                out: In<Slot<'_, $ident<$($types),*>>, R>,
            ) {
                munge!(let $ident($($types,)*) = out);
                $(
                    self.$indices.emplace($types);
                )*
            }
        }
    }
}

/// A type alias for the unit type.
pub type RelTuple0 = ();

define_tuple!(1, RelTuple1<TA>, (EA), 0,);
define_tuple!(
    2,
    RelTuple2<TA, TB>,
    (EA, EB),
    0, 1,
);
define_tuple!(
    3,
    RelTuple3<TA, TB, TC>,
    (EA, EB, EC),
    0, 1, 2,
);
define_tuple!(
    4,
    RelTuple4<TA, TB, TC, TD>,
    (EA, EB, EC, ED),
    0, 1, 2, 3,
);
define_tuple!(
    5,
    RelTuple5<TA, TB, TC, TD, TE>,
    (EA, EB, EC, ED, EE),
    0, 1, 2, 3, 4,
);
define_tuple!(
    6,
    RelTuple6<TA, TB, TC, TD, TE, TF>,
    (EA, EB, EC, ED, EE, EF),
    0, 1, 2, 3, 4, 5,
);
define_tuple!(
    7,
    RelTuple7<TA, TB, TC, TD, TE, TF, TG>,
    (EA, EB, EC, ED, EE, EF, EG),
    0, 1, 2, 3, 4, 5, 6,
);
define_tuple!(
    8,
    RelTuple8<TA, TB, TC, TD, TE, TF, TG, TH>,
    (EA, EB, EC, ED, EE, EF, EG, EH),
    0, 1, 2, 3, 4, 5, 6, 7,
);
define_tuple!(
    9,
    RelTuple9<TA, TB, TC, TD, TE, TF, TG, TH, TI>,
    (EA, EB, EC, ED, EE, EF, EG, EH, EI),
    0, 1, 2, 3, 4, 5, 6, 7, 8,
);
define_tuple!(
    10,
    RelTuple10<TA, TB, TC, TD, TE, TF, TG, TH, TI, TJ>,
    (EA, EB, EC, ED, EE, EF, EG, EH, EI, EJ),
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
);
define_tuple!(
    11,
    RelTuple11<TA, TB, TC, TD, TE, TF, TG, TH, TI, TJ, TK>,
    (EA, EB, EC, ED, EE, EF, EG, EH, EI, EJ, EK),
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
);
define_tuple!(
    12,
    RelTuple12<TA, TB, TC, TD, TE, TF, TG, TH, TI, TJ, TK, TL>,
    (EA, EB, EC, ED, EE, EF, EG, EH, EI, EJ, EK, EL),
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
);
define_tuple!(
    13,
    RelTuple13<TA, TB, TC, TD, TE, TF, TG, TH, TI, TJ, TK, TL, TM>,
    (EA, EB, EC, ED, EE, EF, EG, EH, EI, EJ, EK, EL, EM),
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
);
define_tuple!(
    14,
    RelTuple14<TA, TB, TC, TD, TE, TF, TG, TH, TI, TJ, TK, TL, TM, TN>,
    (EA, EB, EC, ED, EE, EF, EG, EH, EI, EJ, EK, EL, EM, EN),
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
);
define_tuple!(
    15,
    RelTuple15<TA, TB, TC, TD, TE, TF, TG, TH, TI, TJ, TK, TL, TM, TN, TO>,
    (EA, EB, EC, ED, EE, EF, EG, EH, EI, EJ, EK, EL, EM, EN, EO),
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
);
define_tuple!(
    16,
    RelTuple16<TA, TB, TC, TD, TE, TF, TG, TH, TI, TJ, TK, TL, TM, TN, TO, TP>,
    (EA, EB, EC, ED, EE, EF, EG, EH, EI, EJ, EK, EL, EM, EN, EO, EP),
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
);
