use crate::{ Entity, EntityMap, ComponentSet, ArchetypeMap, Archetype };

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
    pub fn spawn<T: ComponentSet>(&mut self, cmp: T) -> Entity
    {
        // alloc a new entity ID
        let ent = Entity::next(1).start;

        // get or create archetype
        let arch = self.archetypes.get_or_insert::<T>();

        // insert entity into archetype
        let loc = arch.insert(ent.id());

        // insert components into archetype
        cmp.insert(arch, loc);

        // cache entity location
        self.entities.insert(ent, loc);

        // return the entity
        ent
    }

    /// get an entity archetype within this scene, if it exists
    pub fn archetype<T: ComponentSet>(&self) -> Option<&Archetype>
    {
        self.archetypes.get::<T>()
    } 
}