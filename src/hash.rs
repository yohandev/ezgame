use std::hash::{ Hash, BuildHasher, Hasher };
use std::collections::hash_map::RandomState;
use std::borrow::Borrow;

use hashbrown::raw::RawTable;

/// marks this type's `Hash` implementation equal to that of `T`.
/// that is, if `foo` is of type `T`, and `self == foo`, then
/// `hash(self) == hash(foo)`. violating this rule while still
/// implementing the `Hashlike` trait is undefined behaviour.
pub trait Hashlike<T: Hash>
{
    /// hash `self` as if it were a `T`. see the [`Hash`] trait's [`hash`]
    ///
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    /// [`hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html#method.hash
    fn hash<H: Hasher>(&self, state: &mut H);

    /// hash `&[Self]` as if it were a `&[T]`. see the [`Hash`] trait's [`hash_slice`]
    ///
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    /// [`hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html#method.hash_slice
    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H) where Self: Sized,
    {
        for piece in data
        {
            piece.hash(state);
        }
    }
}

/// [`HashMap`] implementation that uses the `Hashlike` trait on top of
/// the [`Borrow`] trait, granting additional flexibility.
///
/// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
/// [`Borrow`]: https://doc.rust-lang.org/std/borrow/trait.Borrow.html
pub struct HashlikeMap<K, V>
{
    table: RawTable<(K, V)>,
    hasher: RandomState,
}

/// any type that implements `Hash` can, obviously hash like itself
impl<T: Hash> Hashlike<T> for T
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        // use the normal hash implementation
        Hash::hash(self, state);
    }
}

impl<K, V> HashlikeMap<K, V>
{
    /// see [`new`]
    ///
    /// [`new`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.new
    pub fn new() -> Self
    {
        Self::default()
    }

    /// see [`capacity`]
    ///
    /// [`capacity`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.capacity
    pub fn capacity(&self) -> usize
    {
        self.table.capacity()
    }

    /// see [`len`]
    ///
    /// [`len`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.len
    pub fn len(&self) -> usize
    {
        self.table.len()
    }
}

impl<K: Hash + Eq, V> HashlikeMap<K, V>
{
    #[inline]
    pub fn get_like<Q>(&self, k: &Q) -> Option<&V> where
        Q: Hashlike<K> + PartialEq<K> + ?Sized
    {
        // Avoid `Option::map` because it bloats LLVM IR.
        match self.get_like_key_value(k)
        {
            Some((_, v)) => Some(v),
            None => None,
        }
    }

    pub fn get_key_value<Q: ?Sized>(&self, k: &Q) -> Option<(&K, &V)> where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let hash = get_hash(&self.hasher, k);

        // Avoid `Option::map` because it bloats LLVM IR.
        match self.table.find(hash, |x| k.eq(x.0.borrow())) {
            Some(item) => unsafe {
                let &(ref key, ref value) = item.as_ref();
                Some((key, value))
            }
            None => None,
        }
    }

    #[inline]
    pub fn get_like_key_value<Q>(&self, k: &Q) -> Option<(&K, &V)> where Q: Hashlike<K> + PartialEq<K> + ?Sized
    {
        let hash = get_hash(&self.hasher, k);

        // Avoid `Option::map` because it bloats LLVM IR.
        match self.table.find(hash, |x| k.eq(&x.0))
        {
            Some(item) => unsafe
            {
                let &(ref key, ref value) = item.as_ref();
                Some((key, value))
            }
            None => None,
        }
    }

    #[inline]
    pub fn get_like_key_value_mut<Q>(&mut self, k: &Q) -> Option<(&K, &mut V)> where Q: Hashlike<K> + PartialEq<K> + ?Sized
    {
        let hash = get_hash(&self.hasher, k);

        // Avoid `Option::map` because it bloats LLVM IR.
        match self.table.find(hash, |x| k.eq(&x.0))
        {
            Some(item) => unsafe {
                let &mut (ref key, ref mut value) = item.as_mut();
                Some((key, value))
            }
            None => None,
        }
    }

    pub fn get_like_mut<Q>(&mut self, k: &Q) -> Option<&mut V> where Q: Hashlike<K> + PartialEq<K> + ?Sized
    {
        let hash = get_hash(&self.hasher, k);

        // Avoid `Option::map` because it bloats LLVM IR.
        match self.table.find(hash, |x| k.eq(&x.0))
        {
            Some(item) => Some(unsafe { &mut item.as_mut().1 }),
            None => None,
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    {
        unsafe
        {
            let hash = get_hash(&self.hasher, &k);

            if let Some(item) = self.table.find(hash, |x| k.eq(&x.0))
            {
                Some(std::mem::replace(&mut item.as_mut().1, v))
            }
            else
            {
                let hasher = &self.hasher;
                self.table.insert(hash, (k, v), |x| get_hash(hasher, &x.0));

                None
            }
        }
    }
}

impl<K: Clone, V: Clone> Clone for HashlikeMap<K, V>
{
    fn clone(&self) -> Self
    {
        Self
        {
            hasher: self.hasher.clone(),
            table: self.table.clone(),
        }
    }
}

impl<K, V> Default for HashlikeMap<K, V>
{
    fn default() -> Self
    {
        Self
        {
            hasher: RandomState::new(),
            table: RawTable::new(),
        }
    }
}

/// hashes a value
fn get_hash<K: Hash, Q: Hashlike<K> + ?Sized>(hash_builder: &impl BuildHasher, val: &Q) -> u64
{
    let mut state = hash_builder.build_hasher();
    val.hash(&mut state);
    state.finish()
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[derive(Debug, Eq, PartialEq, Hash)]
    struct Foo(i32);
    #[derive(Debug, Hash)]
    struct Bar(Foo);

    impl PartialEq<Foo> for Bar
    {
        fn eq(&self, other: &Foo) -> bool
        {
            self.0.eq(other)
        }
    }

    impl Hashlike<Foo> for Bar
    {
        fn hash<H: Hasher>(&self, state: &mut H)
        {
            Hash::hash(&self.0, state)
        }
    }

    #[test]
    fn test_hashlike()
    {
        let mut map = HashlikeMap::<Vec<Foo>, String>::new();

        println!("get vec![Foo(2), Foo(5)]      -> {:?}", map.get(&vec![Foo(2), Foo(5)]));
        println!("get (Bar(Foo(2))) -> {:?}", map.get(&vec![Bar(Foo(2)), Bar(Foo(5))]));

        // println!("insert(Foo(2))   -> {:?}", map.insert(Foo(2), "Hello!".to_string()));

        // println!("get(Foo(2))      -> {:?}", map.get(&Foo(2)));
        // println!("get(Bar(Foo(2))) -> {:?}", map.get(&Bar(Foo(2))));
    }
}