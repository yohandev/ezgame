use std::sync::atomic::{ AtomicU64, Ordering };
use std::ops::{ Range, Index };
use std::collections::HashMap;
use std::fmt::Display;

/// unique identifier for an entity(64bit integer)
///
/// obtained from `Scene::spawn` and can be stored for
/// future reference. since `2^64` is such a large number,
/// entities should never need to be recycled.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Entity
{
    id: EntId
}

/// alias for u64, which is used as an entity's identifier
pub type EntId = u64;

/// next entity ID(thread-safe)
static ENT_CURSOR: AtomicU64 = AtomicU64::new(0);

/// structure that maps entity IDs to their component archetype in
/// a "double hashmap" like structure
#[derive(Debug, Default)]
pub struct EntityMap
{
    chunks: HashMap<EntId, EntityMapChunk>
}

/// the storage location of an entity's components
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct EntityLocation
{
    archetype: usize,
    chunk: usize,
    index: usize,
}

/// a chunk within an entity map
///
/// it keeps track of how many entity locations aren't `NULL`,
/// to be removed when `len` is `map.size()`
#[derive(Debug)]
struct EntityMapChunk
{
    map: [EntityLocation; Self::SIZE],
    len: usize
}

impl Entity
{
    /// get this entity's unique identifier
    pub fn id(&self) -> EntId
    {
        self.id
    }

    /// allocate `n` entities and return the range of
    /// their IDs
    pub(crate) fn next(n: u64) -> Range<Entity>
    {
        debug_assert!(n > 0, "cannot allocate 0 entities!");

        let i = ENT_CURSOR.fetch_add(n, Ordering::Relaxed);

        Range
        {
            start: Entity { id: i },
            end: Entity { id: i + n },
        }
    }
}

impl Display for Entity
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_fmt(format_args!("entity#{}", self.id()))
    }
}

impl Index<Entity> for EntityMap
{
    type Output = EntityLocation;

    fn index(&self, index: Entity) -> &Self::Output
    {
        // index of entity within chunk
        let e_ind = index.id() % EntityMapChunk::SIZE as u64;
        // index(key) of chunk
        let c_ind = index.id() - e_ind;

        // has chunk -> maybe entity is in this map?
        if let Some(chunk) = self.chunks.get(&c_ind)
        {
            &chunk.map[e_ind as usize]
        }
        // doesn't have chunk -> entity definitely not here
        else
        {
            &EntityLocation::NULL
        }
    }
}

impl EntityMap
{
    /// insert a new (Entity, Location) pair into the map, or
    /// silently overwrite an existing one
    pub fn insert(&mut self, e: Entity, loc: EntityLocation)
    {
        debug_assert_ne!(loc, EntityLocation::NULL, "cannot insert null location!");

        // index of entity within chunk
        let e_ind = e.id() % EntityMapChunk::SIZE as u64;
        // index(key) of chunk
        let c_ind = e.id() - e_ind;
        // (usize) index of entity within chunk
        let e_ind = e_ind as usize;

        // get chunk(or insert)
        match self.chunks.get_mut(&c_ind)
        {
            Some(chunk) =>
            {
                // insert new...
                if chunk.map[e_ind] == EntityLocation::NULL
                {
                    chunk.len += 1;
                }
                // ...then (re)place
                chunk.map[e_ind] = loc;
            }
            None =>
            {
                // create new chunk...
                let mut chunk = EntityMapChunk::new();

                // ...populate with first location...
                chunk.map[e_ind] = loc;
                chunk.len = 1;

                // ...insert into map
                self.chunks.insert(c_ind, chunk);
            }
        }
    }

    /// remove the (Entity, Location) pair for the given entity
    /// in this map
    pub fn remove(&mut self, e: Entity)
    {
        // index of entity within chunk
        let e_ind = e.id() % EntityMapChunk::SIZE as u64;
        // index(key) of chunk
        let c_ind = e.id() - e_ind;
        // (usize) index of entity within chunk
        let e_ind = e_ind as usize;

        // get chunk
        if let Some(chunk) =  self.chunks.get_mut(&c_ind)
        {
            // check if entity existed...
            if chunk.map[e_ind] != EntityLocation::NULL
            {
                chunk.len -= 1;
            }

            // ...set to null regardless of previous state
            chunk.map[e_ind] = EntityLocation::NULL;

            // remove the chunk if empty
            if chunk.len == 0
            {
                self.chunks.remove(&c_ind);
            }
        }
    }

    /// get the `EntityLocation` from the `(Entity, Location)` pair in
    /// this map. returns `EntityLocation::NULL` if it doesn't contain it
    pub fn get(&self, e: Entity) -> EntityLocation
    {
        // index of entity within chunk
        let e_ind = e.id() % EntityMapChunk::SIZE as u64;
        // index(key) of chunk
        let c_ind = e.id() - e_ind;
        // (usize) index of entity within chunk
        let e_ind = e_ind as usize;

        // get chunk
        match self.chunks.get(&c_ind)
        {
            // chunk exists -> check entity itself
            Some(chunk) => chunk.map[e_ind],
            // chunk doesn't exist -> entity cannot exist
            None => EntityLocation::NULL
        }
    }

    /// does this map contains the entity `e`?
    /// basically, is the entity alive as far as this map knows?
    pub fn contains(&self, e: Entity) -> bool
    {
        self.get(e) != EntityLocation::NULL
    }
}

impl EntityMapChunk
{
    /// number of locations per chunk
    const SIZE: usize = 16;

    fn new() -> Self
    {
        Self
        {
            map: [EntityLocation::NULL; Self::SIZE],
            len: 0,
        }
    }
}

impl EntityLocation
{
    /// represents an null entity location
    pub const NULL: EntityLocation = EntityLocation { archetype: usize::MAX, chunk: 0, index: 0 };

    /// get the archetype ID part of this `EntityLocation`
    pub fn archetype(&self) -> usize
    {
        self.archetype
    }

    /// get the chunk ID within the archetype part of this `EntityLocation`
    pub fn chunk(&self) -> usize
    {
        self.chunk
    }

    /// get the entity index within the chunk within the archetype part of this `EntityLocation`
    pub fn index(&self) -> usize
    {
        self.index
    }

    /// create a new entity location. this is an inner library operation
    pub(crate) fn new(archetype: usize, chunk: usize, index: usize) -> Self
    {
        Self { archetype, chunk, index }
    }
}

impl Display for EntityLocation
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if self == &EntityLocation::NULL
        {
            f.write_fmt(format_args!("null"))
        }
        else
        {
            f.write_fmt(format_args!("archetype#{}[{}]", self.archetype, self.index))
        }
    }
}