use std::collections::HashMap;
use std::cell::UnsafeCell;
use std::alloc::Layout;
use std::ptr::NonNull;
use std::any::TypeId;
use std::rc::Rc;

use crate::{ EntId, EntityLocation, Component, ComponentSet };

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
    free: Vec<usize>,
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

/// meta-data about an arbitrary type
#[derive(Debug, Copy, Clone)]
pub struct TypeMeta
{
    /// store the type ID
    id: TypeId,
    /// size, in bytes, of the type
    size: usize,
    /// alignment, in bytes, of the type
    align: usize,
    /// drop logic needs to be cached to because we're working with u8*
    drop: DropFn,
}

/// pointer to the drop in place function for a type, when it is
/// represented as a void*
type DropFn = unsafe fn(*mut u8);

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
                .fold(0, |acc, n| acc + n.size);
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
                    alloc += (t.align - (alloc % t.align)) % t.align;

                    // add to meta
                    meta.insert(t.id, (t, alloc));
                    
                    // component data(increment alloc_size)
                    alloc += t.size * max;
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
        let free = vec![];

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
        self.free.push(self.chunks.len());
        // append the chunk to this archetype
        self.chunks.push(ArchetypeChunk { meta, data, len });

        // return the new chunk's index
        *self.free.last().unwrap()
    }

    /// inserts an entity into this archetype, and returns the index where it was placed
    /// every type must be written immediately after
    pub(crate) fn insert(&mut self, e: EntId) -> EntityLocation
    {
        // info for the entity location being returned
        let archetype = self.meta.id;
        let chunk = self.free
            .last()
            .copied()
            .unwrap_or_else(|| self.new_chunk());
        let index = self.chunks[chunk].len;

        // insert entity ID
        self.chunks[chunk].entities_mut()[index] = e;

        // increment length
        self.chunks[chunk].len += 1;

        // returns location
        EntityLocation { archetype, chunk, index }
    }

    /// set the component data for an `EntityLocation` inside this archetype
    ///
    /// this should be called directly after `Archetype::insert` and for every
    /// component
    ///
    /// `loc` must be valid and `typeof(T)` must be within this archetype
    pub(crate) fn set<T: Component>(&mut self, loc: EntityLocation, cmp: T)
    {
        self.chunks[loc.chunk].components_mut::<T>()[loc.index] = cmp;
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
        let meta = self.meta.cmp
            .get(&ty)
            .expect("attempting to access components not within this archetype!");

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
        let meta = self.meta.cmp
            .get(&ty)
            .expect("attempting to access components not within this archetype!");

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
        let meta = self.meta.cmp
            .get(&TypeId::of::<T>())
            .expect("attempting to access components not within this archetype!");

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
        let meta = self.meta.cmp
            .get(&TypeId::of::<T>())
            .expect("attempting to access components not within this archetype!");

        // pointer to the start of component being accessed
        let ptr = unsafe { (*self.data.get()).as_ptr().add(meta.1) as *mut T };

        // create slice
        unsafe { std::slice::from_raw_parts_mut(ptr, self.len) }
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
    pub fn get_dyn<F>(&mut self, ty: &Vec<TypeId>, meta: F) -> &mut Archetype
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
    pub fn get<T: ComponentSet>(&mut self) -> &mut Archetype
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

impl TypeMeta
{
    /// get the type meta given a compile-time type
    pub fn of<T: 'static>() -> Self
    {
        unsafe fn drop_ptr<T>(ptr: *mut u8)
        {
            ptr.cast::<T>().drop_in_place()
        }

        Self
        {
            id: TypeId::of::<T>(),
            size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
            drop: drop_ptr::<T>,
        }
    }
}

impl PartialOrd for TypeMeta
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>
    {
        Some(self.cmp(other))
    }
}

impl PartialEq for TypeMeta
{
    fn eq(&self, other: &Self) -> bool
    {
        self.id == other.id
    } 
}

impl Ord for TypeMeta
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering
    {
        self.align
            .cmp(&other.align)                      // compare by alignment
            .reverse()                              // reverse to maximize space
                                                    // start with greatest alignment ->
                                                    // only space wasted is `abs(greatest_align - align_of(EntId))`
            .then_with(|| self.id.cmp(&other.id))   // tie breaker via ID
    }
}

impl Eq for TypeMeta { }