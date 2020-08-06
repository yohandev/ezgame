use crate::Entity;

/// a container for entities and their components.
///
/// responsible for (de)spawning and querying entities which
/// are unique to an application and thus can be moved from
/// scene to scene.
pub struct Scene
{

}

impl Scene
{
    /// create a new epmty scene
    pub fn new() -> Self
    {
        Scene { }
    }

    /// spawn a single entity into this scene with the given
    /// components
    pub fn spawn(&self, _: ()) -> Entity
    {
        Entity::next(1).start
    }
}