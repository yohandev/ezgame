use std::hash::{ Hash, BuildHasher, Hasher };
use std::collections::hash_map::RandomState;
use std::marker::PhantomData;
use std::iter::FusedIterator;
use std::fmt::Debug;

use hashbrown::raw::{ RawIter, RawTable };

/// [`HashMap`] implementation that's unsafe and its only requirement for keys is
/// they implement the [`Hash`] and [`PartialEq`] traits. their [`Hash`] impl
/// must meet the following condition for some `UnsafeMap<K, V>` where
/// `let foo: K = //...`
/// - if `self.eq(foo)` via `PartialEq`, then `hash(self) == hash(foo)`
///
/// this bypasses type contracts, granting additional flexibility, but has no checks
/// against the condition above. failing to meet it is undefined behaviour
///
/// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
/// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
/// [`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
pub struct UnsafeMap<K, V>
{
    table: RawTable<(K, V)>,
    hasher: RandomState,
}

impl<K, V> UnsafeMap<K, V>
{
    /// see [`new`]
    ///
    /// [`new`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.new
    pub fn new() -> Self
    {
        Self::default()
    }

    /// see [`with_capacity`]
    ///
    /// [`with_capacity`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.with_capacity
    pub fn with_capacity(n: usize) -> Self
    {
        Self
        {
            hasher: RandomState::new(),
            table: RawTable::with_capacity(n)
        }
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

    /// see [`iter`]
    ///
    /// [`iter`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.iter
    pub fn iter(&self) -> Iter<'_, K, V>
    {
        // Here we tie the lifetime of self to the iter.
        unsafe
        {
            Iter
            {
                inner: self.table.iter(),
                marker: PhantomData,
            }
        }
    }
}

impl<K: Hash + Eq, V> UnsafeMap<K, V>
{
    /// unrestricted version of [`get`]. see docs on [`UnsafeMap`] for conditions on generic
    /// parameter `Q` that must be met.
    ///
    /// [`get`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.get
    /// [`UnsafeMap`]: struct.UnsafeMap.html
    #[inline]
    pub unsafe fn get<Q>(&self, k: &Q) -> Option<&V> where Q: Hash + PartialEq<K> + ?Sized
    {
        // Avoid `Option::map` because it bloats LLVM IR.
        match self.get_key_value(k)
        {
            Some((_, v)) => Some(v),
            None => None,
        }
    }

    /// unrestricted version of [`get_key_value`]. see docs on [`UnsafeMap`] for conditions
    /// on generic parameter `Q` that must be met.
    ///
    /// [`get_key_value`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.get_key_value
    /// [`UnsafeMap`]: struct.UnsafeMap.html
    #[inline]
    pub unsafe fn get_key_value<Q>(&self, k: &Q) -> Option<(&K, &V)> where Q: Hash + PartialEq<K> + ?Sized
    {
        let hash = get_hash(&self.hasher, k);

        // Avoid `Option::map` because it bloats LLVM IR.
        match self.table.find(hash, |x| k.eq(&x.0))
        {
            Some(item) =>
            {
                let &(ref key, ref value) = item.as_ref();
                Some((key, value))
            }
            None => None,
        }
    }

    /// unrestricted version of [`get_key_value_mut`]. see docs on [`UnsafeMap`] for conditions
    /// on generic parameter `Q` that must be met.
    ///
    /// [`get_key_value_mut`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.get_key_value_mut
    /// [`UnsafeMap`]: struct.UnsafeMap.html
    #[inline]
    pub unsafe fn get_key_value_mut<Q>(&mut self, k: &Q) -> Option<(&K, &mut V)> where Q: Hash + PartialEq<K> + ?Sized
    {
        let hash = get_hash(&self.hasher, k);

        // Avoid `Option::map` because it bloats LLVM IR.
        match self.table.find(hash, |x| k.eq(&x.0))
        {
            Some(item) =>
            {
                let &mut (ref key, ref mut value) = item.as_mut();
                Some((key, value))
            }
            None => None,
        }
    }

    /// unrestricted version of [`get_mut`]. see docs on [`UnsafeMap`] for conditions
    /// on generic parameter `Q` that must be met.
    ///
    /// [`get_mut`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.get_mut
    /// [`UnsafeMap`]: struct.UnsafeMap.html
    pub unsafe fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V> where Q: Hash + PartialEq<K> + ?Sized
    {
        let hash = get_hash(&self.hasher, k);

        // Avoid `Option::map` because it bloats LLVM IR.
        match self.table.find(hash, |x| k.eq(&x.0))
        {
            Some(item) => Some(&mut item.as_mut().1),
            None => None,
        }
    }

    /// see [`insert`]
    ///
    /// [`insert`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.insert
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

    /// see [`contains_key`]
    ///
    /// [`contains_key`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.contains_key
    pub unsafe fn contains_key<Q>(&self, k: &Q) -> bool where Q: Hash + PartialEq<K> + ?Sized
    {
        self.get(k).is_some()
    }

    /// see [`values`]
    ///
    /// [`values`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.values
    pub fn values(&self) -> Values<'_, K, V>
    {
        Values { inner: self.iter() }
    }
}

/// see [`Iter`]
///
/// [`Iter`]: https://doc.rust-lang.org/std/collections/hash_map/struct.Iter.html
#[derive(Clone)]
pub struct Iter<'a, K, V>
{
    inner: RawIter<(K, V)>,
    marker: PhantomData<(&'a K, &'a V)>,
}

/// see [`Values`]
///
/// [`Values`]: https://doc.rust-lang.org/std/collections/hash_map/struct.Values.html
#[derive(Clone)]
pub struct Values<'a, K, V>
{
    inner: Iter<'a, K, V>,
}

impl<K: Clone, V: Clone> Clone for UnsafeMap<K, V>
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

impl<K, V> Default for UnsafeMap<K, V>
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

impl<K: Debug, V: Debug> Debug for UnsafeMap<K, V>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<'a, K, V> IntoIterator for &'a UnsafeMap<K, V>
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V>
    {
        self.iter()
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)>
    {
        // Avoid `Option::map` because it bloats LLVM IR.
        match self.inner.next()
        {
            Some(x) => unsafe
            {
                let r = x.as_ref();
                Some((&r.0, &r.1))
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V>
{
    fn len(&self) -> usize
    {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> {}

impl<'a, K, V> Iterator for Values<'a, K, V>
{
    type Item = &'a V;

    fn next(&mut self) -> Option<&'a V>
    {
        // Avoid `Option::map` because it bloats LLVM IR.
        match self.inner.next()
        {
            Some((_, v)) => Some(v),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        self.inner.size_hint()
    }
}

impl<K, V> ExactSizeIterator for Values<'_, K, V>
{
    fn len(&self) -> usize
    {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for Values<'_, K, V> { }

/// hashes a value
fn get_hash<K: Hash + ?Sized>(hash_builder: &impl BuildHasher, val: &K) -> u64
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

    #[test]
    fn test_hashlike()
    {
        let mut map = UnsafeMap::<Vec<Foo>, String>::new();

        println!("get vec![Foo(2), Foo(5)]           -> {:?}", unsafe { map.get(&vec![Foo(2), Foo(5)]) });
        println!("get vec![Bar(Foo(2)), Bar(Foo(5))] -> {:?}", unsafe { map.get(&vec![Bar(Foo(2)), Bar(Foo(5))]) });

        println!("insert(Foo(2))                     -> {:?}", map.insert(vec![Foo(2), Foo(5)], "Hello!".to_string()));

        println!("get vec![Foo(2), Foo(5)]           -> {:?}", unsafe { map.get(&vec![Foo(2), Foo(5)]) });
        println!("get vec![Bar(Foo(2)), Bar(Foo(5))] -> {:?}", unsafe { map.get(&vec![Bar(Foo(2)), Bar(Foo(5))]) });
    }
}