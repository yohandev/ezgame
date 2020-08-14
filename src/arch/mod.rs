// mod
mod chunk;
mod meta;

use self::chunk::*;
use self::meta::*;

// Archetype
use std::collections::HashSet;
use std::rc::Rc;

use super::{ CmpMeta, Entity, EntityLocation };

// collection of a specific combination of components
#[derive(Debug)]
pub struct Archetype
{
    /// meta-data about this `Archetype`
    pub(self) meta: Rc<ArchetypeMeta>,
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
            meta: Rc::new(ArchetypeMeta::new(id, types)),
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
        let chunk_id = self.free
            .iter()
            .next()
            .copied()
            .unwrap_or_else(|| ArchetypeChunk::append_to(self));
        let chunk = &mut self.chunks[chunk_id];
        let index = chunk.len;

        // increment length
        chunk.len += 1;

        // chunk is full
        if chunk.len == self.meta.max
        {
            self.free.remove(&chunk_id);
        }

        // insert entity ID
        chunk.entities_mut()[index] = e;

        // returns location
        EntityLocation::new(archetype, chunk_id, index)
    }
}