mod data;

use ::criterion::black_box;
use ::mischief::{GhostRef, In, Region, Slot, StaticToken};
use ::munge::munge;
use ::rand::Rng;
use ::rel_alloc::{alloc::RelAllocator, EmplaceIn, RelString, RelVec};
use ::rel_core::{Emplace, EmplaceExt, Move, Portable, U16, U64};
use ::rel_slab_allocator::{RelSlabAllocator, SlabAllocator};
use ::rel_util::Align16;
use ::situ::{alloc::RawRegionalAllocator, DropRaw};

use crate::{from_data::FromData, gen::generate_vec};

#[derive(DropRaw, Move, Portable)]
#[repr(C)]
pub struct RelAddress {
    pub x0: u8,
    pub x1: u8,
    pub x2: u8,
    pub x3: u8,
}

unsafe impl<R: Region> Emplace<RelAddress, R> for &'_ data::Address {
    fn emplaced_meta(&self) -> <RelAddress as ptr_meta::Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelAddress>, R>,
    ) {
        munge!(
            let RelAddress {
                x0,
                x1,
                x2,
                x3,
            } = out;
        );

        self.x0.emplace(x0);
        self.x1.emplace(x1);
        self.x2.emplace(x2);
        self.x3.emplace(x3);
    }
}

#[derive(DropRaw, Move, Portable)]
#[repr(C)]
pub struct RelEntry<A: RawRegionalAllocator> {
    pub address: RelAddress,
    pub identity: RelString<A>,
    pub userid: RelString<A>,
    pub date: RelString<A>,
    pub request: RelString<A>,
    pub code: U16,
    pub size: U64,
}

unsafe impl<A, R> Emplace<RelEntry<A>, R::Region>
    for FromData<'_, R, data::Entry>
where
    A: DropRaw + RawRegionalAllocator<Region = R::Region>,
    R: Clone + RelAllocator<A>,
{
    fn emplaced_meta(&self) -> <RelEntry<A> as ptr_meta::Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelEntry<A>>, A::Region>,
    ) {
        use ::rel_alloc::string;

        munge!(
            let RelEntry {
                address,
                identity,
                userid,
                date,
                request,
                code,
                size,
            } = out;
        );

        let Self { alloc, data } = self;

        data.address.emplace(address);
        string::Clone(alloc.clone(), &data.identity).emplace(identity);
        string::Clone(alloc.clone(), &data.userid).emplace(userid);
        string::Clone(alloc.clone(), &data.date).emplace(date);
        string::Clone(alloc.clone(), &data.request).emplace(request);
        data.code.emplace(code);
        data.size.emplace(size);
    }
}

#[derive(DropRaw, Move, Portable)]
#[repr(C)]
pub struct RelLog<A: RawRegionalAllocator> {
    pub entries: RelVec<RelEntry<A>, A>,
}

unsafe impl<A, R> Emplace<RelLog<A>, R::Region> for FromData<'_, R, data::Log>
where
    A: DropRaw + Move<R::Region> + RawRegionalAllocator<Region = R::Region>,
    R: Clone + RelAllocator<A>,
{
    fn emplaced_meta(&self) -> <RelLog<A> as ptr_meta::Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelLog<A>>, A::Region>,
    ) {
        use ::rel_alloc::vec;

        munge!(let RelLog { entries } = out);

        let entries =
            vec::WithCapacity(self.alloc.clone(), self.data.entries.len())
                .emplace_mut(entries);

        RelVec::extend(
            In::into_inner(entries),
            self.data.entries.iter().map(|data| FromData {
                alloc: self.alloc.clone(),
                data,
            }),
        );
    }
}

fn populate_buffer(data: &data::Log, buffer: Slot<'_, [u8]>) -> usize {
    StaticToken::acquire(|mut token| {
        let alloc =
            SlabAllocator::<_>::try_new_in(buffer, GhostRef::leak(&mut token))
                .unwrap();

        let log = FromData { alloc, data }
            .emplace_in::<RelLog<RelSlabAllocator<_>>>(alloc);

        alloc.deposit(log);
        alloc.shrink_to_fit()
    })
}

pub fn make_bench(
    rng: &mut impl Rng,
    input_size: usize,
) -> impl FnMut() -> usize {
    let input = data::Log {
        entries: generate_vec(rng, input_size),
    };

    let mut bytes = Align16::frame(10_000_000);

    move || {
        black_box(populate_buffer(
            black_box(&input),
            black_box(bytes.slot().as_bytes()),
        ))
    }
}
