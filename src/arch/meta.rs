use std::collections::HashMap;
use std::alloc::Layout;

use crate::{ CmpId, CmpMeta, Entity };
use super::ArchetypeChunk;

/// meta-data about an archetype, which is shared(via `Rc`) between a parent `Archetype`
/// and its `ArchetypeChunk` children. this is caclulated once and never altered in
/// the `Archetype::new` constructor
#[derive(Debug)]
pub struct ArchetypeMeta
{
    /// index of this archetype in the `Scene`'s archetype vector
    id: usize,
    /// (meta-data, offset) about the components' types stored in this archetype
    cmp: HashMap<CmpId, (CmpMeta, usize)>,
    /// (cached) max entities that can be stored in a single chunk within
    /// this archetype
    ///
    /// a chunk stores the exact same amount of components between varying
    /// types, with no overlap inside roughly 16kb
    max: usize,
    /// (cached) layout for every chunk allocations for this archetype
    layout: Layout,
}

impl ArchetypeMeta
{
    /// create a new archetype meta from a sorted vector of component meta
    pub fn new(id: usize, types: &Vec<CmpMeta>) -> Self
    {
        // assert types are sorted
        debug_assert!
        (
            types
                .windows(2)
                .all(|n| n[0] < n[1]),
            "component meta is unsorted or contains duplicates!"
        );

        // alignment of chunks is that of Entity, because `*self.data.get()` starts
        // with entity IDs
        let align = std::mem::align_of::<Entity>();

        // size, in bytes, of all components + ID for one entity excluding padding
        let size = std::mem::size_of::<Entity>() + types
            .iter()
            .fold(0, |acc, n| acc + n.size());
        // max entities that can be stored in this chunk
        let max = ArchetypeChunk::TARGET_SIZE / size;
        // `alloc`: size, in bytes, of the allocation per chunk. it over-allocates slightly
        // to have space for padding, but ends up roughly equal to `16kb`
        // `meta`: meta info about the components within this archetype
        let (alloc, cmp) =
        {
            // iterate components, incrementing allocation size
            // start with entity IDs
            let mut alloc = std::mem::size_of::<Entity>() * max;
            // meta will have exact same size as `types` argument
            let mut meta = HashMap::with_capacity(types.len());

            for t in types
            {
                // padding for alignment(increment alloc_size)
                alloc += (t.alignment() - (alloc % t.alignment())) % t.alignment();

                // add to meta
                meta.insert(t.id(), (t.clone(), alloc));
                
                // component data(increment alloc_size)
                alloc += t.size() * max;
            }

            (alloc, meta)
        };

        // layout for a chunk allocation within this archetype
        let layout = Layout::from_size_align(alloc, align).unwrap();

        // return the archetype meta...
        ArchetypeMeta { id, cmp, max, layout }
    }
}