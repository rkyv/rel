(function() {var implementors = {
"rel_alloc":[["impl&lt;T:&nbsp;<a class=\"trait\" href=\"rel_core/basis/trait.BasisPointee.html\" title=\"trait rel_core::basis::BasisPointee\">BasisPointee</a>&lt;B&gt; + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>, A:&nbsp;<a class=\"trait\" href=\"situ/alloc/regional/trait.RawRegionalAllocator.html\" title=\"trait situ::alloc::regional::RawRegionalAllocator\">RawRegionalAllocator</a>, B:&nbsp;<a class=\"trait\" href=\"rel_core/basis/trait.Basis.html\" title=\"trait rel_core::basis::Basis\">Basis</a>&gt; <a class=\"trait\" href=\"rel_core/portable/trait.Portable.html\" title=\"trait rel_core::portable::Portable\">Portable</a> for <a class=\"struct\" href=\"rel_alloc/boxed/struct.RelBox.html\" title=\"struct rel_alloc::boxed::RelBox\">RelBox</a>&lt;T, A, B&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"rel_core/rel_ptr/struct.RelPtr.html\" title=\"struct rel_core::rel_ptr::RelPtr\">RelPtr</a>&lt;T, A::<a class=\"associatedtype\" href=\"situ/alloc/regional/trait.RawRegionalAllocator.html#associatedtype.Region\" title=\"type situ::alloc::regional::RawRegionalAllocator::Region\">Region</a>, B&gt;: <a class=\"trait\" href=\"rel_core/portable/trait.Portable.html\" title=\"trait rel_core::portable::Portable\">Portable</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;A: <a class=\"trait\" href=\"rel_core/portable/trait.Portable.html\" title=\"trait rel_core::portable::Portable\">Portable</a>,</span>"],["impl&lt;A:&nbsp;<a class=\"trait\" href=\"situ/alloc/regional/trait.RawRegionalAllocator.html\" title=\"trait situ::alloc::regional::RawRegionalAllocator\">RawRegionalAllocator</a>, B:&nbsp;<a class=\"trait\" href=\"rel_core/basis/trait.Basis.html\" title=\"trait rel_core::basis::Basis\">Basis</a>&gt; <a class=\"trait\" href=\"rel_core/portable/trait.Portable.html\" title=\"trait rel_core::portable::Portable\">Portable</a> for <a class=\"struct\" href=\"rel_alloc/string/struct.RelString.html\" title=\"struct rel_alloc::string::RelString\">RelString</a>&lt;A, B&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"rel_alloc/vec/struct.RelVec.html\" title=\"struct rel_alloc::vec::RelVec\">RelVec</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.66.0/std/primitive.u8.html\">u8</a>, A, B&gt;: <a class=\"trait\" href=\"rel_core/portable/trait.Portable.html\" title=\"trait rel_core::portable::Portable\">Portable</a>,</span>"],["impl&lt;T, A:&nbsp;<a class=\"trait\" href=\"situ/alloc/regional/trait.RawRegionalAllocator.html\" title=\"trait situ::alloc::regional::RawRegionalAllocator\">RawRegionalAllocator</a>, B:&nbsp;<a class=\"trait\" href=\"rel_core/basis/trait.Basis.html\" title=\"trait rel_core::basis::Basis\">Basis</a>&gt; <a class=\"trait\" href=\"rel_core/portable/trait.Portable.html\" title=\"trait rel_core::portable::Portable\">Portable</a> for <a class=\"struct\" href=\"rel_alloc/vec/struct.RelVec.html\" title=\"struct rel_alloc::vec::RelVec\">RelVec</a>&lt;T, A, B&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"rel_core/rel_ptr/struct.RelPtr.html\" title=\"struct rel_core::rel_ptr::RelPtr\">RelPtr</a>&lt;T, A::<a class=\"associatedtype\" href=\"situ/alloc/regional/trait.RawRegionalAllocator.html#associatedtype.Region\" title=\"type situ::alloc::regional::RawRegionalAllocator::Region\">Region</a>, B&gt;: <a class=\"trait\" href=\"rel_core/portable/trait.Portable.html\" title=\"trait rel_core::portable::Portable\">Portable</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;B::<a class=\"associatedtype\" href=\"rel_core/basis/trait.Basis.html#associatedtype.Usize\" title=\"type rel_core::basis::Basis::Usize\">Usize</a>: <a class=\"trait\" href=\"rel_core/portable/trait.Portable.html\" title=\"trait rel_core::portable::Portable\">Portable</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;B::<a class=\"associatedtype\" href=\"rel_core/basis/trait.Basis.html#associatedtype.Usize\" title=\"type rel_core::basis::Basis::Usize\">Usize</a>: <a class=\"trait\" href=\"rel_core/portable/trait.Portable.html\" title=\"trait rel_core::portable::Portable\">Portable</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;A: <a class=\"trait\" href=\"rel_core/portable/trait.Portable.html\" title=\"trait rel_core::portable::Portable\">Portable</a>,</span>"]],
"rel_core":[],
"rel_slab_allocator":[["impl&lt;'a, U:&nbsp;<a class=\"trait\" href=\"mischief/unique/trait.Unique.html\" title=\"trait mischief::unique::Unique\">Unique</a>, B1:&nbsp;<a class=\"trait\" href=\"rel_core/basis/trait.Basis.html\" title=\"trait rel_core::basis::Basis\">Basis</a>, B2:&nbsp;<a class=\"trait\" href=\"rel_core/basis/trait.Basis.html\" title=\"trait rel_core::basis::Basis\">Basis</a>&gt; <a class=\"trait\" href=\"rel_core/portable/trait.Portable.html\" title=\"trait rel_core::portable::Portable\">Portable</a> for <a class=\"struct\" href=\"rel_slab_allocator/struct.RelSlabAllocator.html\" title=\"struct rel_slab_allocator::RelSlabAllocator\">RelSlabAllocator</a>&lt;'a, U, B1, B2&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"rel_core/rel_ref/struct.RelRef.html\" title=\"struct rel_core::rel_ref::RelRef\">RelRef</a>&lt;'a, SlabControl&lt;U, B1&gt;, <a class=\"struct\" href=\"rel_slab_allocator/struct.SlabRegion.html\" title=\"struct rel_slab_allocator::SlabRegion\">SlabRegion</a>&lt;U&gt;, B2&gt;: <a class=\"trait\" href=\"rel_core/portable/trait.Portable.html\" title=\"trait rel_core::portable::Portable\">Portable</a>,</span>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()