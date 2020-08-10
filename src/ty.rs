use std::any::TypeId;

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

    /// get the `TypeId` of this type
    pub fn id(&self) -> TypeId
    {
        self.id
    }

    /// get the size, in bytes, of the type
    pub fn size(&self) -> usize
    {
        self.size
    }

    /// get the alignment, in bytes, of the type
    pub fn alignment(&self) -> usize
    {
        self.align
    }

    /// drop in place a void pointer, assuming the pointer
    /// points to the same type referenced by this `TypeMeta`
    pub unsafe fn drop(&self, ptr: *mut u8)
    {
        (self.drop)(ptr)
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