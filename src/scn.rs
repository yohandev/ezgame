use crate::{ EntityMap, Entity, ArchetypeMap, CmpSet };

/// a container for entities and their components.
///
/// responsible for (de)spawning and querying entities which
/// are unique to an application and thus can be moved from
/// scene to scene.
#[derive(Debug, Default)]
pub struct Scene
{
    entities: EntityMap,
    archetypes: ArchetypeMap,
}

impl Scene
{
    /// spawn a single entity into this scene with the given
    /// components
    pub fn spawn(&mut self, cmp: impl CmpSet) -> Entity
    {
        // alloc a new entity ID
        let ent = Entity::next(1).start;

        // get or create archetype
        let arch = self.archetypes.get_or_insert(&cmp);

        // insert entity into archetype
        let loc = arch.insert(ent);

        // insert components into archetype
        //cmp.insert(arch, loc);

        // cache entity location
        self.entities.insert(ent, loc);

        // return the entity
        ent
    }
}

impl std::fmt::Display for Scene
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Scene:\n{}", self.entities)
    }
}