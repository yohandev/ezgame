use std::collections::HashMap;

use crate::{ CmpId, CmpSet };
use super::Archetype;

/// structure that maps component `Vec<TypeMeta>` to component archetypes in
/// a hashmap-like structure
#[derive(Debug, Default)]
pub struct ArchetypeMap
{
    /// complete list of `Archetype`s. the collection can be expanded but is
    /// never shrunk, therefore elements are 'pinned' and an index can safely
    /// reference an archetype
    arch: Vec<Archetype>,
    /// maps sorted `Vec<CmpId>` to an archetype index in `self.arch`
    map: HashMap<Vec<CmpId>, usize>,
}

impl ArchetypeMap
{
    /// see `ArchetypeMap::get_or_insert`
    ///
    /// both `types` and the output of `meta` MUST be sorted via their `Ord` traits,
    /// similar to implementing the `ComponentSet` trait on a concrete type
    pub fn get_or_insert(&mut self, set: &impl CmpSet) -> &mut Archetype
    {
        let id = set.types(|types| match self.map.get_mut(types)
        {
            Some(i) => *i,
            None =>
            {
                // ID of the new archetype
                let id = self.arch.len();

                // create new archetype
                self.map.insert(Vec::from(types), id);
                self.arch.push(Archetype::new(id, &set.metas()));

                // return ID of the new archetype
                id
            }
        });

        &mut self.arch[id]
    }
}