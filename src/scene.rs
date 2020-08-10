use crate::{ Entity, EntityMap, EntityLocation, Component, ComponentSet, ArchetypeMap, Archetype };

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

    /// is the entity alive and in this `Scene`?
    pub fn contains(&self, ent: Entity) -> bool
    {
        self.entities.contains(ent)
    }

    /// get the `impl Component` for the given entity, if it has
    /// it
    pub fn get<T: Component>(&self, ent: Entity) -> Option<&T>
    {
        // entity location
        let loc = self.entities.get(ent);

        // entity isn't alive
        if loc == EntityLocation::NULL
        {
            None
        }
        else
        {
            // get the chunk, which won't error even if the entity doesn't have
            // the component as long as the entity location is valid
            let chunk = &self.archetypes
                .inner()[loc.archetype] // get the entity's archetype
                .chunks()[loc.chunk];   // get the entity's chunk
            
            // prevent panic, if the entity doesn't have the component
            if chunk.meta().contains::<T>()
            {
                Some(&chunk.components::<T>()[loc.index])
            }
            else
            {
                None
            }
        }
    }

    /// get the `impl Component` for the given entity, if it has
    /// it
    pub fn get_mut<T: Component>(&mut self, ent: Entity) -> Option<&mut T>
    {
        // entity location
        let loc = self.entities.get(ent);

        // entity isn't alive
        if loc == EntityLocation::NULL
        {
            None
        }
        else
        {
            // get the chunk, which won't error even if the entity doesn't have
            // the component as long as the entity location is valid
            let chunk = &mut self.archetypes
                .inner_mut()[loc.archetype] // get the entity's archetype
                .chunks_mut()[loc.chunk];   // get the entity's chunk
            
            // prevent panic, if the entity doesn't have the component
            if chunk.meta().contains::<T>()
            {
                Some(&mut chunk.components_mut::<T>()[loc.index])
            }
            else
            {
                None
            }
        }
    }

    /// does this `Entity` have the component?
    pub fn has<T: Component>(&self, ent: Entity) -> bool
    {
        self.get::<T>(ent).is_some()
    }

    /// get an entity archetype within this scene, if it exists
    pub fn archetype<T: ComponentSet>(&self) -> Option<&Archetype>
    {
        self.archetypes.get::<T>()
    }
}