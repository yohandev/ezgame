use std::sync::atomic::{ AtomicU64, Ordering };
use std::fmt::Display;
use std::ops::Range;

/// unique identifier for an entity(64bit integer)
///
/// obtained from `Scene::spawn` and can be stored for
/// future reference. since `2^64` is such a large number,
/// entities should never need to be recycled.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Entity
{
    id: u64
}

/// next entity ID(thread-safe)
static ENT_CURSOR: AtomicU64 = AtomicU64::new(0);

impl Entity
{
    /// get this entity's unique identifier
    pub fn id(&self) -> u64
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