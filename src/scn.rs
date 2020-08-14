use crate::EntityMap;

/// a container for entities and their components.
///
/// responsible for (de)spawning and querying entities which
/// are unique to an application and thus can be moved from
/// scene to scene.
#[derive(Debug, Default)]
pub struct Scene
{
    entities: EntityMap
}

impl std::fmt::Display for Scene
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Scene:\n{}", self.entities)
    }
}