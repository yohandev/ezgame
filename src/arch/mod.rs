// mod
mod chunk;
mod meta;

use self::chunk::*;
use self::meta::*;

// Archetype
use std::collections::HashSet;

use super::{ CmpMeta, Entity, EntityLocation };

// collection of a specific combination of components
#[derive(Debug)]
pub struct Archetype
{
    /// meta-data about this `Archetype`
    pub(self) meta: ArchetypeMeta,
    /// all chunks in this archetype. the collection can be expanded but is
    /// never shrunk, therefore elements are 'pinned' and an index can safely
    /// reference a chunk
    pub(self) chunks: Vec<ArchetypeChunk>,
    /// list of chunk indices with free entity slots and zero shared components
    ///
    /// TODO: shared component to free chunk map of type `HashMap<..., Vec<ArchetypeChunkIndex>>
    pub(self) free: HashSet<usize>,
}

impl Archetype
{
    /// create a new archetype from a sorted vector of component meta
    pub(crate) fn new(id: usize, types: &Vec<CmpMeta>) -> Self
    {
        Self
        {
            meta: ArchetypeMeta::new(id, types),
            chunks: Default::default(),
            free: Default::default(),
        }
    }

    /// inserts an entity into this archetype, and returns the index where it was placed
    /// every type must be written immediately after
    pub(crate) fn insert(&mut self, e: Entity) -> EntityLocation
    {
        // info for the entity location being returned
        let archetype = self.meta.id;
        let chunk = self.free
            .iter()
            .next()
            .copied()
            .unwrap_or_else(|| ArchetypeChunk::append_to(self));
        let index = self.chunks[chunk].len;

        // increment length
        self.chunks[chunk].len += 1;

        // chunk is full
        if self.chunks[chunk].len == self.meta.max
        {
            self.free.remove(&chunk);
        }

        // insert entity ID
        self.entities_mut(chunk)[index] = e;

        // returns location
        EntityLocation::new(archetype, chunk, index)
    }

    // internal version of `ArchetypeChunkRef::entities_mut`
    pub(self) fn entities_mut(&mut self, chunk_id: usize) -> &mut [Entity]
    {
        // pointer to the start of entity IDs
        let ptr = unsafe { (*self.chunks[chunk_id].data.get()).as_ptr() as *mut Entity };

        // create slice
        unsafe { std::slice::from_raw_parts_mut(ptr, self.chunks[chunk_id].len) }
    }
}