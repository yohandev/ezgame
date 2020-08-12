use crate::{ Archetype, EntityLocation, Type };

/// non-shared component trait
pub trait Component: Send + Sync + 'static { }

// /// shared component trait
// pub trait SharedComponent: Send + Sync + Eq + 'static { }

/// a tuple of non-duplicate, arbitrarily ordered `Component` types
/// and `SharedComponent` types
pub trait ComponentSet
{
    /// returns a sorted vector of the types inside this component set
    ///
    /// uses:
    /// - finding an archetype, based off the hash of the slice returned
    /// - creating an archetype, using the meta inside the slice
    ///
    /// it's highly important that the slice is sorted via `Type`'s `Ord`
    /// trait implementation(reverse-alignment then type id)
    fn ty() -> Vec<Type>;

    /// insert the `Component` data from this set into the archetype at the given
    /// `EntityLocation`
    fn insert(self, arch: &mut Archetype, loc: EntityLocation);
}

/// implements the `ComponentSet` trait for tuples of `impl Component`
macro_rules! impl_component_set
{
    ({$($name:ident),*}, {$($num:tt),*}) =>
    {
        impl<$($name: Component),*> ComponentSet for ($($name,)*)
        {
            fn ty() -> Vec<Type>
            {
                // complete list of types in arbitrary order
                let mut ty = vec![$(Type::of::<$name>()),*];

                // sort
                ty.sort_unstable();

                // return sorted vector
                ty
            }

            #[allow(unused_variables)]
            fn insert(self, arch: &mut Archetype, loc: EntityLocation)
            {
                $(*arch.get_mut(loc).unwrap() = self.$num;)*
            }
        }
    };
}

// pyramid of doom...
impl_component_set!({}, {});
impl_component_set!({ A }, { 0 });
impl_component_set!({ A, B }, { 0, 1 });
impl_component_set!({ A, B, C }, { 0, 1, 2 });
impl_component_set!({ A, B, C, D }, { 0, 1, 2, 3 });
impl_component_set!({ A, B, C, D, E }, { 0, 1, 2, 3, 4 });
impl_component_set!({ A, B, C, D, E, F }, { 0, 1, 2, 3, 4, 5 });
impl_component_set!({ A, B, C, D, E, F, G }, { 0, 1, 2, 3, 4, 5, 6 });
impl_component_set!({ A, B, C, D, E, F, G, H }, { 0, 1, 2, 3, 4, 5, 6, 7 });
impl_component_set!({ A, B, C, D, E, F, G, H, I }, { 0, 1, 2, 3, 4, 5, 6, 7, 8 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M, N }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24 });
impl_component_set!({ A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z }, { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 25 });