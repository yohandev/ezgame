use ezgame::*;

#[test]
fn entity_loc_one_chunk()
{
    let scene = Scene::default();
    let mut map = EntityMap::default();

    let ent = scene.spawn(());

    map.insert(ent, EntityLocation { archetype: 1, index: 0 });

    println!("{} is at {}", ent, map[ent]);

    println!("removing location for {}", ent);
    map.remove(ent);

    println!("{} is now at {}", ent, map[ent]);
    println!("map should be empty(no chunks): {:?}", map);
}

#[test]
fn entity_loc_more_chunks()
{
    let scene = Scene::default();
    let mut map = EntityMap::default();

    for i in 0..64
    {
        let e = scene.spawn(());

        map.insert(e, EntityLocation { archetype: 1, index: i });

        println!("{} is at {}", e, map[e]);
    }
}

#[test]
#[should_panic]
fn insert_null_loc()
{
    let scene = Scene::default();
    let mut map = EntityMap::default();

    map.insert(scene.spawn(()), EntityLocation::NULL);
}