use std::cell::UnsafeCell;
use std::ptr::NonNull;
use std::rc::Rc;

use super::ArchetypeMeta;

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

impl ArchetypeChunk
{
    /// target size, in bytes, of a single chuk within an archetype(16kb)
    pub const TARGET_SIZE: usize = 16_000;
}