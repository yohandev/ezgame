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

    /// despawns a single entity if it's alive and in this scene.
    /// returns whether the operation was succesful
    pub fn despawn(&mut self, ent: Entity) -> bool
    {
        // entity location
        let loc = self.entities.get(ent);

        // entity isn't alive
        if loc == EntityLocation::NULL
        {
            false
        }
        else
        {
            // get the entity's current archetype
            let arch = &mut self.archetypes.inner_mut()[loc.archetype()];
            
            // update the location of the entity that was moved, if any
            if let Some(moved) = arch.remove(loc, true)
            {
                self.entities.insert(unsafe { Entity::from_id(moved) }, loc);
            }

            // remove entity from scene
            self.entities.remove(ent);

            true
        }
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
            // `Archetype::get` returns an option already
            self.archetypes.inner()[loc.archetype()].get(loc)
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
            // `Archetype::get_mut` returns an option already
            self.archetypes.inner_mut()[loc.archetype()].get_mut(loc)
        }
    }

    /// does this `Entity` have the component?
    pub fn has<T: Component>(&self, ent: Entity) -> bool
    {
        self.get::<T>(ent).is_some()
    }

    /// add components to an entity, if it exists
    /// returns whether the operation was succesful
    pub fn add<T: ComponentSet>(&mut self, ent: Entity, cmp: T) -> bool
    {
        // entity location
        let old_loc = self.entities.get(ent);

        // entity isn't alive
        if old_loc == EntityLocation::NULL
        {
            false
        }
        else
        {
            // begin borrow old archetype...
            let (metas, types, old_cmp) =
            {
                // get the entity's current archetype
                let old_arch = &mut self.archetypes.inner_mut()[old_loc.archetype()];

                // sums the entity's current type[meta] + that of the components being added,
                // with no overlap
                let mut metas = old_arch
                    .meta()
                    .component_metas()
                    .map(|n| *n)
                    .collect::<Vec<_>>();
                let mut types = old_arch
                    .meta()
                    .component_types()
                    .map(|n| *n)
                    .collect::<Vec<_>>();

                // go through types being added
                for ty in T::meta()
                {
                    // if adding an existing component, override
                    if let Some(ptr) = old_arch.get_mut_dyn(old_loc, ty.id())
                    {
                        // drop old component
                        unsafe { ty.drop(ptr); }
                    }
                    // entity didn't have component, add it to list
                    else
                    {
                        metas.push(ty);
                        types.push(ty.id());
                    }
                }
                // needs to sort, since we're merging two `ComponentSet`s
                metas.sort();
                types.sort();

                // vector of old components being moved
                // (TypeMeta, *const u8) pair vector
                let old_cmp = old_arch
                    .meta()
                    .component_metas()
                    .map(|ty| (*ty, old_arch.get_dyn(old_loc, ty.id()).unwrap()))
                    .collect::<Vec<_>>();

                // used later in the function...
                (metas, types, old_cmp)
            };

            // get or create archetype where the entity will live
            let new_arch = self.archetypes.get_or_insert_dyn(&types, || (&metas).clone());

            // insert entity into new archetype
            let new_loc = new_arch.insert(ent.id());

            // move *all* old components
            for (ty, src) in old_cmp
            {
                let dst = new_arch.get_mut_dyn(new_loc, ty.id()).unwrap();
                
                unsafe { std::ptr::copy_nonoverlapping(src, dst, ty.size()); }
            }

            // insert the new components too
            cmp.insert(new_arch, new_loc);

            // delete from old archetype and update the location of the entity
            // that was moved, if any
            if let Some(moved) = self.archetypes.inner_mut()[old_loc.archetype()].remove(old_loc, false)
            {
                self.entities.insert(unsafe { Entity::from_id(moved) }, old_loc);
            }

            // update entity location
            self.entities.insert(ent, new_loc);

            true
        }
    }

    /// get an entity archetype within this scene, if it exists
    pub fn archetype<T: ComponentSet>(&self) -> Option<&Archetype>
    {
        self.archetypes.get::<T>()
    }
}

impl std::fmt::Display for Scene
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Scene:\n{}", self.entities)
    }
}