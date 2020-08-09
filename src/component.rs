use std::any::TypeId;

use crate::TypeMeta;

/// non-shared component trait
pub trait Component: Send + Sync + 'static { }

/// shared component trait
pub trait SharedComponent: Send + Sync + Eq + 'static { }

/// a tuple of non-duplicate, arbitrarily ordered `Component` types
/// and `SharedComponent` types
pub trait ComponentSet
{
    fn meta() -> Vec<TypeMeta>;
}

/// identifier unique to a type of
/// component
pub struct ComponentId
{
    id: TypeId,
    #[cfg(debug_assertions)]
    name: &'static str
}

impl ComponentId
{
    pub fn of<T: Component>() -> Self
    {
        Self
        {
            id: TypeId::of::<T>(),
            #[cfg(debug_assertions)]
            name: std::any::type_name::<T>(),
        }
    }
}