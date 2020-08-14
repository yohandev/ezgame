/// a statically-defined, non-shared component
///
/// this trait should absolutely *not* be implemented manually,
/// and must rather use `#[derive(Component)]`
pub trait Component: Sync + Send + Sized + 'static
{
    /// unique identifier for this type of component
    const ID: CmpId;

    /// meta-data about this component type
    const META: CmpMeta = CmpMeta
    {
        id: Self::ID,
        size: std::mem::size_of::<Self>() as u32,
        align: std::mem::align_of::<Self>() as u32,
        drop: drop_ptr::<Self>
    };
}

/// meta-data about a component type, rust-compiled or dynamic
#[derive(Debug, Clone)]
pub struct CmpMeta
{
    /// component ID generated via the `Component` derive
    id: CmpId,
    /// size, in bytes, of the type
    size: u32,
    /// alignment, in bytes, of the type
    align: u32,
    /// destructor function ptr
    drop: DropFn,
}

/// unique identifer for a component type, rust-compiled or dynamic
///
/// new instances should be obtained from `Component::id` exclusively
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CmpId(u64);

/// function pointer to drop a certain type, given a void ptr.
/// it's wrapped in an option because some types don't need to be
/// dropped.
pub type DropFn = unsafe fn(*mut u8);

/// drops a certain type given a void ptr. used in the `Component::META`
/// constant, as it is a `DropFn` type
#[allow(dead_code)]
unsafe fn drop_ptr<T>(ptr: *mut u8)
{
    ptr.cast::<T>().drop_in_place()
}

impl CmpId
{
    /// creates a new component ID instance from its inner u64. this should
    /// only be called by the `#[derive(Component)]` implementation, hence why
    /// it's unsafe.
    #[allow(dead_code)]
    pub const unsafe fn from_u64(n: u64) -> Self
    {
        Self(n)
    }
}

impl CmpMeta
{
    /// get this component type's unique identifier
    #[inline]
    pub fn id(&self) -> CmpId
    {
        self.id
    }

    /// get this component type's size, in bytes
    #[inline]
    pub fn size_u32(&self) -> u32
    {
        self.size
    }

    /// get this component type's alignment, in bytes
    #[inline]
    pub fn alignment_u32(&self) -> u32
    {
        self.align
    }

    /// get this component type's size, in bytes
    #[inline]
    pub fn size(&self) -> usize
    {
        self.size as usize
    }

    /// get this component type's alignment, in bytes
    #[inline]
    pub fn alignment(&self) -> usize
    {
        self.align as usize
    }
}

impl PartialOrd for CmpMeta
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>
    {
        self.id.partial_cmp(&other.id)
    }
}

impl PartialEq for CmpMeta
{
    fn eq(&self, other: &Self) -> bool
    {
        self.id.eq(&other.id)
    }
}

impl Ord for CmpMeta
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering
    {
        self.id.cmp(&other.id)
    }
}

impl Eq for CmpMeta { }

impl PartialOrd<CmpId> for CmpMeta
{
    fn partial_cmp(&self, other: &CmpId) -> Option<std::cmp::Ordering>
    {
        self.id.partial_cmp(other)
    }
}

impl PartialEq<CmpId> for CmpMeta
{
    fn eq(&self, other: &CmpId) -> bool
    {
        self.id.eq(other)
    }
}