use std::cell::UnsafeCell;
use std::alloc::Layout;
use std::ptr::NonNull;

use super::Archetype;

/// a single, 16kb chunk in an archetype
#[derive(Debug)]
pub struct ArchetypeChunk
{
    /// ~16kb chunk of packed `EntId` + `impl Component`
    ///
    /// `*data.get()[0]` is the first entity ID, therefore, `data.get()`
    /// is a pointer aligned to `EntId`
    ///
    /// the rest of `*(data + ...)` is either more IDs or component data, aligned:
    /// - `[EntId, EntId, EntId, ~, ~, *, A, A, A, ~, ~, *, *, B, B, B, ~, ~]`
    ///     - `EntId` = some entity ID
    ///     - `A` = some component data A
    ///     - `B` = some component data B
    ///     - `~` = free space
    ///     - `*` = padding for alignment
    pub(super) data: UnsafeCell<NonNull<u8>>,
    /// number of entities currently stored in this chunk
    pub(super) len: usize,
}

impl ArchetypeChunk
{
    /// target size, in bytes, of a single chuk within an archetype(16kb)
    pub const TARGET_SIZE: usize = 16_000;

    // create a new chunk aligned to its parent archetype, then append it.
    // returns the chunk's index
    pub(super) fn append_to(arch: &mut Archetype) -> usize
    {
        // first get a well-aligned layout
        let layout = arch.meta.layout;
        // make a heap allocation and get the pointer
        let ptr = unsafe
        {
            std::alloc::alloc(layout).cast::<u8>()
        };
        // make a cell out of the pointer
        let data = UnsafeCell::new(NonNull::new(ptr).unwrap());

        // chunk starts empty(no entities)
        let len = 0;

        // mark the new chunk as free(which it will be)
        arch.free.insert(arch.chunks.len());
        // append the chunk to the archetype
        arch.chunks.push(ArchetypeChunk { data, len });

        // return the new chunk's index
        arch.chunks.len() - 1
    }
}