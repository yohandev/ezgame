use std::collections::{ HashMap, HashSet };
use std::cell::UnsafeCell;
use std::alloc::Layout;
use std::ptr::NonNull;
use std::any::TypeId;
use std::rc::Rc;

use crate::{ EntId, EntityLocation, Component, ComponentSet, TypeMeta };

#[derive(Debug)]
pub struct Archetype
{
    /// meta-data about this `Archetype`, which is shared with its `ArchetypeChunk`
    /// children
    meta: Rc<ArchetypeMeta>,
    /// all chunks in this archetype. the collection can be expanded but is
    /// never shrunk, therefore elements are 'pinned' and an index can safely
    /// reference a chunk
    chunks: Vec<ArchetypeChunk>,
    /// list of chunk indices with free entity slots and zero shared components
    ///
    /// TODO: shared component to free chunk map of type `HashMap<..., Vec<ArchetypeChunkIndex>>
    free: HashSet<usize>,
}

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
    data: UnsafeCell<NonNull<u8>>,
    /// number of entities currently stored in this chunk
    len: usize,
}

/// meta-data about an archetype, which is shared(via `Rc`) between a parent `Archetype`
/// and its `ArchetypeChunk` children. this is caclulated once and never altered in
/// the `Archetype::new` constructor
#[derive(Debug)]
pub struct ArchetypeMeta
{
    /// index of this archetype in the `Scene`'s archetype vector
    id: usize,
    /// meta-data about the components' types stored in this archetype
    cmp: HashMap<TypeId, CmpMeta>,
    /// (cached) max entities that can be stored in a single chunk within
    /// this archetype
    ///
    /// a chunk stores the exact same amount of components between varying
    /// types, with no overlap inside roughly 16kb
    max: usize,
    /// (cached) layout for every chunk allocations for this archetype
    layout: Layout,
}

/// structure that maps component `Vec<TypeMeta>` to component archetypes in
/// a hashmap-like structure
#[derive(Debug, Default)]
pub struct ArchetypeMap
{
    /// complete list of `Archetype`s. the collection can be expanded but is
    /// never shrunk, therefore elements are 'pinned' and an index can safely
    /// reference an archetype
    arch: Vec<Archetype>,
    /// maps sorted `Vec<TypeMeta>` to an archetype index in `self.arch`
    map: HashMap<Vec<TypeId>, usize>,
}

/// meta-data about an arbitrary component type
///
/// tuple of (`meta`: TypeMeta, `offset`: usize)
///
/// `meta`: info about the component's type
///
/// `offset`: offset, in bytes, of where this type begins in a chunk. given a chunk:
/// - - `[EntId, EntId, EntId, ~, ~, *, A, A, A, ~, ~, *, *, B, B, B, ~, ~]`
///     - `EntId` = some entity ID
///     - `A` = some component data A
///     - `B` = some component data B
///     - `~` = free space
///     - `*` = padding for alignment
///
/// component `A` has offset of `6` and B of `13`. the chunk can accomodate
/// for exactly n amounts of A components and n amounts of B components, no more
/// no less. offset considers the type's alignments too
type CmpMeta = (TypeMeta, usize);

impl Archetype
{
    /// target size, in bytes, of a chunk within this archetype(16kb)
    pub const CHUNK_SIZE: usize = 16_000;

    /// creates a new archetype from a list of (maybe unsorted) types
    fn new(id: usize, ty: Vec<TypeMeta>) -> Self
    {
        // archetype meta...
        let meta =
        {
            // sort types by alignment(greatest to least)...
            // note: already comes in sorted
            //ty.sort_unstable();

            // alignment of chunks(EntId, because `*self.data.get()` starts with entity IDs
            let align = std::mem::align_of::<EntId>();

            // size, in bytes, of all components + EntId for one entity excluding padding
            let size = std::mem::size_of::<EntId>() + ty
                .iter()
                .fold(0, |acc, n| acc + n.size());
            // max entities that can be stored in this chunk
            let max = Self::CHUNK_SIZE / size;
            // `alloc`: size, in bytes, of the allocation per chunk. it over-allocates slightly
            // to have space for padding, but ends up roughly equal to `16kb`
            // `meta`: meta info about the components within this archetype
            let (alloc, cmp) =
            {
                // incrementing allocation size...
                // ...start with entity IDs
                let mut alloc = std::mem::size_of::<EntId>() * max;
                // meta...
                // ...will have exact same size as `ty` argument
                let mut meta = HashMap::with_capacity(ty.len());

                for t in ty
                {
                    // padding for alignment(increment alloc_size)
                    alloc += (t.alignment() - (alloc % t.alignment())) % t.alignment();

                    // add to meta
                    meta.insert(t.id(), (t, alloc));
                    
                    // component data(increment alloc_size)
                    alloc += t.size() * max;
                }

                (alloc, meta)
            };

            // layout for a chunk allocation within this archetype
            let layout = Layout::from_size_align(alloc, align).unwrap();

            // return the archetype meta...
            ArchetypeMeta { id, cmp, max, layout }
        };

        // use a shared ref, to share with children chunks
        let meta = Rc::new(meta);
        // start with no chunks...
        let chunks = vec![];
        // ...no chunks therefore no free chunks
        let free = HashSet::new();

        Self { meta, chunks, free }
    }

    /// create a new empty chunk with no shared components and add it to
    /// this archetype. returns the chunk's index
    fn new_chunk(&mut self) -> usize
    {
        // first get a well-aligned layout
        let layout = self.meta.layout;
        // make a heap allocation and get the pointer
        let ptr = unsafe
        {
            std::alloc::alloc(layout).cast::<u8>()
        };
        // make a cell out of the pointer
        let data = UnsafeCell::new(NonNull::new(ptr).unwrap());

        // clone the shared ref to the parent `Archetype`'s meta
        let meta = Rc::clone(&self.meta);

        // chunk starts empty(no entities)
        let len = 0;

        // mark the new chunk as free(which it will be)
        self.free.insert(self.chunks.len());
        // append the chunk to this archetype
        self.chunks.push(ArchetypeChunk { meta, data, len });

        // return the new chunk's index
        self.chunks.len() - 1
    }

    /// inserts an entity into this archetype, and returns the index where it was placed
    /// every type must be written immediately after
    pub(crate) fn insert(&mut self, e: EntId) -> EntityLocation
    {
        // info for the entity location being returned
        let archetype = self.meta.id;
        let chunk = self.free
            .iter()
            .next()
            .copied()
            .unwrap_or_else(|| self.new_chunk());
        let index = self.chunks[chunk].len;

        // increment length
        self.chunks[chunk].len += 1;

        // chunk is full
        if self.chunks[chunk].len == self.meta.max
        {
            self.free.remove(&chunk);
        }

        // insert entity ID
        self.chunks[chunk].entities_mut()[index] = e;

        // returns location
        EntityLocation::new(archetype, chunk, index)
    }

    /// remove an entity from this archetype and return the entity whose `EntityLocation`
    /// was altered(if at all)
    ///
    /// chunks in archetypes are kept packed, and the removing an entity means
    /// replacing its slot with the last entity in the chunk
    pub(crate) fn remove(&mut self, loc: EntityLocation, drop: bool) -> Option<EntId>
    {
        debug_assert_eq!(loc.archetype(), self.meta().id, "attempting to remove an entity not in this archetype!");

        // get the chunk being affected
        let chunk = &mut self.chunks[loc.chunk()];

        // is the entity being removed *not* the last?
        // yes -> place the last entity in the entity being removed
        // no -> do nothing
        // returns the EntId of the entity that was moved(previous last entity in chunk)
        let moved = if loc.index() != chunk.len - 1
        {
            // swap out EntIds
            let last_ent =
            {
                let ent = chunk.entities_mut();
                let last = *ent.last().unwrap();

                // slot of the entity being removed is now the last entity
                ent[loc.index()] = last;

                // the last entity is returned because its location was moved
                last
            };

            // swap out components, one-by-one
            for (_, (ty, offset)) in &chunk.meta().cmp
            {
                let (src, dst) = unsafe
                {
                    // pointer to the start of the component region
                    let ptr = (*chunk.data.get()).as_ptr().add(*offset);

                    // component of the entity being deleted
                    let dst = ptr.add(loc.index() * ty.size());

                    // component of the last entity in the chunk
                    let src = ptr.add((chunk.len - 1) * ty.size());

                    (src, dst)
                };

                // first, we drop the old component
                if drop
                {
                    unsafe { ty.drop(dst); }
                }
                
                // copy the last entity's component into the slot where the
                // entity being deleted was
                //
                // if C is being deleted, G takes its place via memcpy  
                //        ⬇⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺|
                // [A, B, C, D, E, F, G]
                unsafe { std::ptr::copy_nonoverlapping(src, dst, ty.size()); }
            }

            Some(last_ent)
        }
        else
        {
            None
        };

        // decrease length of chunk
        chunk.len -= 1;

        // chunk now has a free spot
        self.free.insert(loc.chunk());

        // return the entity moved(which is now at the provided loc)
        moved
    }

    /// set the component data for an `EntityLocation` inside this archetype
    ///
    /// this should be called directly after `Archetype::insert` and for every
    /// component
    ///
    /// `loc` must be valid and `typeof(T)` must be within this archetype
    pub fn set<T: Component>(&mut self, loc: EntityLocation, cmp: T)
    {
        debug_assert_eq!(self.meta.id, loc.archetype(), "attempting to access entity location outside this archetype!");

        self.chunks[loc.chunk()].components_mut::<T>()[loc.index()] = cmp;
    }

    /// see `Archetype::set`
    ///
    /// the _dyn flavor of functions is for scripting languages, where runtime
    /// types are used
    pub fn set_dyn(&mut self, loc: EntityLocation, ty: &TypeMeta, cmp: &[u8])
    {
        debug_assert_eq!(self.meta.id, loc.archetype(), "attempting to access entity location outside this archetype!");
        debug_assert_eq!(ty.size(), cmp.len(), "attempting to set to a component of the wrong size!");

        let (src, dst) = unsafe
        {
            // pointer to entity location `loc` for the component type `ty`
            let dst = self.chunks[loc.chunk()]
                .components_mut_dyn(ty.id(), ty.size())
                .as_mut_ptr()
                .add(loc.index() * ty.size());

            // source is just the input `cmp`
            let src = cmp.as_ptr();

            (src, dst)
        };
        
        unsafe { std::ptr::copy_nonoverlapping(src, dst, ty.size()); }
    }

    /// get the chunks within this archetype
    pub fn chunks(&self) -> &[ArchetypeChunk]
    {
        &self.chunks
    }

    /// get the chunks within this archetype
    pub fn chunks_mut(&mut self) -> &mut [ArchetypeChunk]
    {
        &mut self.chunks
    }

    /// get the meta for this `Archetype` or `ArchetypeChunk`(they're the same)
    pub fn meta(&self) -> &ArchetypeMeta
    {
        &self.meta
    }
}

impl ArchetypeChunk
{
    /// see `ArchetypeChunk::components`
    ///
    /// the _dyn flavor of functions is for scripting languages, where runtime
    /// types are used
    pub fn components_dyn(&self, ty: TypeId, size: usize) -> &[u8]
    {
        // meta-data about the component being accessed
        let meta = self.meta.get_dyn(ty);

        // pointer to the start of component being accessed
        let ptr = unsafe { (*self.data.get()).as_ptr().add(meta.1) };

        // create slice
        unsafe { std::slice::from_raw_parts(ptr, size * self.len) }
    }

    /// see `ArchetypeChunk::components`
    ///
    /// the _dyn flavor of functions is for scripting languages, where runtime
    /// types are used
    pub fn components_mut_dyn(&mut self, ty: TypeId, size: usize) -> &mut [u8]
    {
        // meta-data about the component being accessed
        let meta = self.meta.get_dyn(ty);

        // pointer to the start of component being accessed
        let ptr = unsafe { (*self.data.get()).as_ptr().add(meta.1) };

        // create slice
        unsafe { std::slice::from_raw_parts_mut(ptr, size * self.len) }
    }

    /// returns a slice of components within this `ArchetypeChunk`. the `T` parameter
    /// must `impl Component` AND be stored within this archetype, else the function
    /// will panic.
    ///
    /// the slice returned only contains the occupied components, not the entire capacity:
    /// `&[T].len() == chunk.len()`
    pub fn components<T: Component>(&self) -> &[T]
    {
        // meta-data about the component being accessed
        let meta = self.meta.get::<T>();

        // pointer to the start of component being accessed
        let ptr = unsafe { (*self.data.get()).as_ptr().add(meta.1) as *const T };

        // create slice
        unsafe { std::slice::from_raw_parts(ptr, self.len) }
    }

    /// returns a slice of components within this `ArchetypeChunk`. the `T` parameter
    /// must `impl Component` AND be stored within this archetype, else the function
    /// will panic.
    ///
    /// the slice returned only contains the occupied components, not the entire capacity:
    /// `&[T].len() == chunk.len()`
    pub fn components_mut<T: Component>(&mut self) -> &mut [T]
    {
        // meta-data about the component being accessed
        let meta = self.meta.get::<T>();

        // pointer to the start of component being accessed
        let ptr = unsafe { (*self.data.get()).as_ptr().add(meta.1) as *mut T };

        // create slice
        unsafe { std::slice::from_raw_parts_mut(ptr, self.len) }
    }

    /// see `ArchetypeChunk::components`
    ///
    /// the _dyn flavor of functions is for scripting languages, where runtime
    /// types are used
    ///
    /// the try_ flavor of functions returns an option instead of panicking
    pub fn try_components_dyn(&self, ty: TypeId, size: usize) -> Option<&[u8]>
    {
        // meta-data about the component being accessed
        let meta = self.meta.try_get_dyn(ty)?;

        // pointer to the start of component being accessed
        let ptr = unsafe { (*self.data.get()).as_ptr().add(meta.1) };

        // create slice
        Some(unsafe { std::slice::from_raw_parts(ptr, size * self.len) })
    }

    /// see `ArchetypeChunk::components`
    ///
    /// the _dyn flavor of functions is for scripting languages, where runtime
    /// types are used
    ///
    /// the try_ flavor of functions returns an option instead of panicking
    pub fn try_components_mut_dyn(&mut self, ty: TypeId, size: usize) -> Option<&mut [u8]>
    {
        // meta-data about the component being accessed
        let meta = self.meta.try_get_dyn(ty)?;

        // pointer to the start of component being accessed
        let ptr = unsafe { (*self.data.get()).as_ptr().add(meta.1) };

        // create slice
        Some(unsafe { std::slice::from_raw_parts_mut(ptr, size * self.len) })
    }

    /// see `ArchetypeChunk::components`
    ///
    /// the try_ flavor of functions returns an option instead of panicking
    pub fn try_components<T: Component>(&self) -> Option<&[T]>
    {
        // meta-data about the component being accessed
        let meta = self.meta.try_get::<T>()?;

        // pointer to the start of component being accessed
        let ptr = unsafe { (*self.data.get()).as_ptr().add(meta.1) as *const T };

        // create slice
        Some(unsafe { std::slice::from_raw_parts(ptr, self.len) })
    }

    /// see `ArchetypeChunk::components`
    ///
    /// the try_ flavor of functions returns an option instead of panicking
    pub fn try_components_mut<T: Component>(&mut self) -> Option<&mut [T]>
    {
        // meta-data about the component being accessed
        let meta = self.meta.try_get::<T>()?;

        // pointer to the start of component being accessed
        let ptr = unsafe { (*self.data.get()).as_ptr().add(meta.1) as *mut T };

        // create slice
        Some(unsafe { std::slice::from_raw_parts_mut(ptr, self.len) })
    }

    /// returns a slice of entity IDs within this chunk. the slice returned only contains the
    /// occupied entity slots, not the entire capacity: `&[EntId].len() == chunk.len()`
    pub fn entities(&self) -> &[EntId]
    {
        // pointer to the start of entity IDs
        let ptr = unsafe { (*self.data.get()).as_ptr() as *const EntId };

        // create slice
        unsafe { std::slice::from_raw_parts(ptr, self.len) }
    }

    /// returns a slice of entity IDs within this chunk. the slice returned only contains the
    /// occupied entity slots, not the entire capacity: `&[EntId].len() == chunk.len()`
    pub fn entities_mut(&mut self) -> &mut [EntId]
    {
        // pointer to the start of entity IDs
        let ptr = unsafe { (*self.data.get()).as_ptr() as *mut EntId };

        // create slice
        unsafe { std::slice::from_raw_parts_mut(ptr, self.len) }
    }

    /// get the meta for this `Archetype` or `ArchetypeChunk`(they're the same)
    pub fn meta(&self) -> &ArchetypeMeta
    {
        &self.meta
    }
}

impl ArchetypeMap
{
    /// see `ArchetypeMap::get`
    ///
    /// both `ty` and the output of `meta` MUST be sorted via their `Ord` traits,
    /// similar to implementing the `ComponentSet` trait on a concrete type
    ///
    /// the _dyn flavor of functions is for scripting languages, where runtime
    /// types are used
    pub fn get_or_insert_dyn<F>(&mut self, ty: &Vec<TypeId>, meta: F) -> &mut Archetype
        where F: FnOnce() -> Vec<TypeMeta>
    {
        let id = match self.map.get_mut(ty)
        {
            Some(i) => *i,
            None =>
            {
                // ID of the new archetype
                let id = self.arch.len();

                // create new archetype
                self.arch.push(Archetype::new(id, meta()));
                self.map.insert(ty.clone(), id);

                // return ID of the new archetype
                id
            }
        };

        &mut self.arch[id]
    }

    /// get an archetype or insert it into `self`
    pub fn get_or_insert<T: ComponentSet>(&mut self) -> &mut Archetype
    {
        // get the `TypeId`s within the set
        let ty = T::ty();

        let id = match self.map.get_mut(&ty)
        {
            Some(i) => *i,
            None =>
            {
                // ID of the new archetype
                let id = self.arch.len();

                // create new archetype
                self.arch.push(Archetype::new(id, T::meta()));
                self.map.insert(ty, id);

                // return ID of the new archetype
                id
            }
        };

        &mut self.arch[id]
    }

    /// get an archetype if it exists
    pub fn get<T: ComponentSet>(&self) -> Option<&Archetype>
    {
        // get the `TypeId`s within the set
        let ty = T::ty();

        // get the archetype if it's there
        self.map
            .get(&ty)
            .map(|id| &self.arch[*id])
    }

    /// get all the `Archetype`s within this map, in order of their
    /// IDs
    ///
    /// this is useful to get an archetype by its ID:
    /// ```rust
    /// let loc: EntityLocation = ...;
    /// 
    /// // get the archetype by its ID
    /// let arch = map.archetypes()[loc.archetype];
    /// ```
    pub fn inner(&self) -> &[Archetype]
    {
        &self.arch
    }

    /// get all the `Archetype`s within this map, in order of their
    /// IDs
    ///
    /// this is useful to get an archetype by its ID:
    /// ```rust
    /// let loc: EntityLocation = ...;
    /// 
    /// // get the archetype by its ID
    /// let arch = map.archetypes()[loc.archetype];
    /// ```
    pub fn inner_mut(&mut self) -> &mut [Archetype]
    {
        &mut self.arch
    }
}

impl ArchetypeMeta
{
    /// see `ArchetypeMeta::contains`
    ///
    /// the _dyn flavor of functions is for scripting languages, where runtime
    /// types are used
    pub fn contains_dyn(&self, id: TypeId) -> bool
    {
        self.cmp.contains_key(&id)
    }

    /// does this archetype contain the `Component`'s `TypeId`?
    pub fn contains<T: Component>(&self) -> bool
    {
        self.contains_dyn(TypeId::of::<T>())
    }

    /// see `ArchetypeMeta::get`
    ///
    /// the _dyn flavor of functions is for scripting languages, where runtime
    /// types are used
    pub fn get_dyn(&self, id: TypeId) -> &CmpMeta
    {
        self.cmp
            .get(&id)
            .expect("attempting to access components not within this archetype!")
    }

    /// get the component meta from a `Component` `TypeId`
    ///
    /// the component must be in this chunk, or the function will panic.
    /// use `ArchetypeMeta::contains` to check
    pub fn get<T: Component>(&self) -> &CmpMeta
    {
        self.get_dyn(TypeId::of::<T>())
    }

    /// see `ArchetypeMeta::try_get`
    ///
    /// the _dyn flavor of functions is for scripting languages, where runtime
    /// types are used
    pub fn try_get_dyn(&self, id: TypeId) -> Option<&CmpMeta>
    {
        self.cmp.get(&id)
    }

    /// try and get the component meta from a `Component` `TypeId`
    ///
    /// the component must be in this chunk, or the function will return none.
    /// alternative to `ArchetypeMeta::get`, which just panics
    pub fn try_get<T: Component>(&self) -> Option<&CmpMeta>
    {
        self.try_get_dyn(TypeId::of::<T>())
    }

    /// returns a copy of the `TypeMeta`s and `TypeId`s within this `ArchetypeMeta`,
    /// in arbitrary order
    ///
    /// this is basically like collecting the (key, value) pairs in this meta
    pub fn types(&self) -> (Vec<TypeId>, Vec<TypeMeta>)
    {
        let key = self.cmp
            .keys()
            .map(|ty| *ty)
            .collect();
        let val = self.cmp
            .values()
            .map(|cmp_meta| cmp_meta.0)
            .collect();

        (key, val)
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