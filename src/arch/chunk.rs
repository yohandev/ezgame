use std::cell::UnsafeCell;
use std::ptr::NonNull;
use std::rc::Rc;

use super::{ Archetype, ArchetypeMeta };
use crate::Entity;

/// a single, 16kb chunk in an archetype
#[derive(Debug)]
pub struct ArchetypeChunk
{
    /// meta-data about this chunk's parent `Archetype`, which is shared with
    /// it too
    meta: Rc<ArchetypeMeta>,
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
        // clone the archetype meta shared reference
        let meta = Rc::clone(&arch.meta);
        // first get a well-aligned layout
        let layout = meta.layout;
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
        arch.chunks.push(ArchetypeChunk { meta, data, len });

        // return the new chunk's index
        arch.chunks.len() - 1
    }

    /// returns a slice of entity IDs within this chunk. the slice returned only contains the
    /// occupied entity slots, not the entire capacity: `&[Entity].len() == chunk.len()`
    pub fn entities(&self) -> &[Entity]
    {
        unsafe
        {
            // pointer to the start of entity IDs
            let ptr = (*self.data.get()).as_ptr() as *const Entity;

            // create slice
            std::slice::from_raw_parts(ptr, self.len)
        }
    }

    /// returns a slice of entity IDs within this chunk. the slice returned only contains the
    /// occupied entity slots, not the entire capacity: `&[Entity].len() == chunk.len()`
    pub fn entities_mut(&mut self) -> &mut [Entity]
    {
        unsafe
        {
            // pointer to the start of entity IDs
            let ptr = (*self.data.get()).as_ptr() as *mut Entity;

            // create slice
            std::slice::from_raw_parts_mut(ptr, self.len)
        }
    }
}

impl Drop for ArchetypeChunk
{
    fn drop(&mut self)
    {
        unsafe
        {
            std::alloc::dealloc((*self.data.get()).as_ptr(), self.meta.layout);
        }
    }
}