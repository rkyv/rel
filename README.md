<p align="center">
    <img src="https://raw.githubusercontent.com/rkyv/rel/main/media/logo_text_color.svg" alt="rel">
</p>
<p align="center">
    rel is an object system based on relative pointers
</p>

_This project is still in its early stages. Development is being done in the open._

## Resources

- [The book](https://rel.rkyv.org) contains information about the project and its direction.
- [Crate docs](https://rel.rkyv.org/docs/rel_core) are available for all crates in the project and
built continuously from the `main` branch.

## What is rel?

rel is the future object model for [rkyv](https://github.com/rkyv/rkyv) and will enable fully
mutable containers and zero-work serialization and deserialization. It achieves this by providing
value emplacement semantics and a full allocator API for immovable data structures. To that end, rel
also encompasses some relatively novel libraries that may be applicable beyond itself.

## What does this mean for rkyv?

Once development on rel is far enough along, rkyv will be replacing its object model (structures
like `ArchivedVec`, `ArchivedBTreeMap`, and more) with rel. rkyv will become focused on providing
users its familiar derive macros and serialization system for easy entry and exit points from rel's
object system. By formally separating the derive system from the object system, it will be easier
for existing users to migrate from rkyv + rel to a fully rel-based project if they want to.

## How can I help?

rel is currently in the experimental phase and needs more thought and scrutiny to help it mature
into a library that rkyv can move on top of. If you're interested in helping shape the future of
rel, [join rkyv's Discord][discord] to discuss the library. Issues are always welcome as well, and
we'll be using them mostly to track feature development as the project moves towards feature
completeness and stability.

If you're already an rkyv user, then you may recognize rel as a core component of rkyv's vision for
the 0.8 release. If you want to know more about how your project may be able to leverage rel, then
[visit the Discord][discord] to discuss the future of rkyv and rel.

[discord]: https://discord.gg/65F6MdnbQh
