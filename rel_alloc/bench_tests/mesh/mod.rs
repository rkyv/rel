mod data;

use ::criterion::black_box;
use ::mischief::{GhostRef, In, Region, Slot, StaticToken};
use ::munge::munge;
use ::rand::Rng;
use ::rel_alloc::{alloc::RelAllocator, EmplaceIn, RelVec};
use ::rel_core::{Emplace, EmplaceExt, Move, Portable, F32};
use ::rel_slab_allocator::{RelSlabAllocator, SlabAllocator};
use ::rel_util::Align16;
use ::situ::{alloc::RawRegionalAllocator, DropRaw};

use crate::{from_data::FromData, gen::generate_vec};

#[derive(DropRaw, Move, Portable)]
#[repr(C)]
pub struct RelVector3 {
    pub x: F32,
    pub y: F32,
    pub z: F32,
}

unsafe impl<R: Region> Emplace<RelVector3, R> for &'_ data::Vector3 {
    fn emplaced_meta(&self) -> <RelVector3 as ptr_meta::Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelVector3>, R>,
    ) {
        munge!(
            let RelVector3 {
                x,
                y,
                z,
            } = out;
        );

        self.x.emplace(x);
        self.y.emplace(y);
        self.z.emplace(z);
    }
}

#[derive(DropRaw, Move, Portable)]
#[repr(C)]
pub struct RelTriangle {
    pub v0: RelVector3,
    pub v1: RelVector3,
    pub v2: RelVector3,
    pub normal: RelVector3,
}

unsafe impl<R: Region> Emplace<RelTriangle, R> for &'_ data::Triangle {
    fn emplaced_meta(&self) -> <RelTriangle as ptr_meta::Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelTriangle>, R>,
    ) {
        munge!(
            let RelTriangle {
                v0,
                v1,
                v2,
                normal,
            } = out;
        );

        self.v0.emplace(v0);
        self.v1.emplace(v1);
        self.v2.emplace(v2);
        self.normal.emplace(normal);
    }
}

#[derive(DropRaw, Move, Portable)]
#[repr(C)]
pub struct RelMesh<A: RawRegionalAllocator> {
    pub triangles: RelVec<RelTriangle, A>,
}

unsafe impl<A, R> Emplace<RelMesh<A>, R::Region> for FromData<'_, R, data::Mesh>
where
    A: DropRaw + RawRegionalAllocator<Region = R::Region>,
    R: RelAllocator<A>,
{
    fn emplaced_meta(&self) -> <RelMesh<A> as ptr_meta::Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelMesh<A>>, A::Region>,
    ) {
        use ::rel_alloc::vec;

        munge!(let RelMesh { triangles } = out);

        let triangles =
            vec::WithCapacity(self.alloc, self.data.triangles.len())
                .emplace_mut(triangles);

        RelVec::extend(In::into_inner(triangles), self.data.triangles.iter());
    }
}

fn populate_buffer(data: &data::Mesh, buffer: Slot<'_, [u8]>) -> usize {
    StaticToken::acquire(|mut token| {
        let alloc =
            SlabAllocator::<_>::try_new_in(buffer, GhostRef::leak(&mut token))
                .unwrap();

        let mesh = FromData { alloc, data }
            .emplace_in::<RelMesh<RelSlabAllocator<_>>>(alloc);

        alloc.deposit(mesh);
        alloc.shrink_to_fit()
    })
}

pub fn make_bench(
    rng: &mut impl Rng,
    input_size: usize,
) -> impl FnMut() -> usize {
    let input = data::Mesh {
        triangles: generate_vec(rng, input_size),
    };

    let mut bytes = Align16::frame(10_000_000);
    move || {
        black_box(populate_buffer(
            black_box(&input),
            black_box(bytes.slot().as_bytes()),
        ))
    }
}
