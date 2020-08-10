use ezgame::*;

#[derive(Debug, Eq, PartialEq)]
struct Pos(i32, i32, i32);
#[derive(Debug, Eq, PartialEq)]
struct Vel(i32, i32, i32);
#[derive(Debug, Eq, PartialEq)]
struct Name(&'static str);

impl Component for Pos { }
impl Component for Vel { }
impl Component for Name { }

#[test]
fn despawn()
{
    fn print_pos_vel(scene: &Scene)
    {
        let chunk = scene
            .archetype::<(Pos, Vel)>()
            .expect("(Pos, Vel) archetype wasn't created!")
            .chunks()
            .first()
            .expect("(Pos, Vel) archetype doesn't have chunks!");

        println!("Ent: {:?}", chunk.entities());
        println!("Pos: {:?}", chunk.components::<Pos>());
        println!("Vel: {:?}", chunk.components::<Vel>());
    }

    let mut scene = Scene::default();

    /* test with one entity */
    let ent0 = scene.spawn((Pos(1, 2, 3), Vel(9, 8, 7)));

    // spawn
    assert!(scene.contains(ent0), "entity#0 wasn't spawned!");
    println!("spawned an entity#0 with Pos(1, 2, 3) and Vel(9, 8, 7)");

    // print
    print_pos_vel(&scene);

    // despawn
    assert!(scene.despawn(ent0), "entity#0 despawn returned false!");

    assert!(!scene.contains(ent0), "entity#0 wasn't despawned!");
    println!("despawned entity#0 with Pos(1, 2, 3) and Vel(9, 8, 7)");

    // print
    print_pos_vel(&scene);

    /* test with two entities */
    let ent1 = scene.spawn((Pos(-1, -2, -3), Vel(5, -10, 18)));
    let ent2 = scene.spawn((Pos(2, 4, 47), Vel(0, 1, -6)));

    // spawn
    assert!(scene.contains(ent1), "entity#1 wasn't spawned!");
    assert!(scene.contains(ent2), "entity#2 wasn't spawned!");
    println!("spawned entity#1 {{ Pos(-1, -2, -3), Vel(5, -10, 18) }} and entity#2 {{ Pos(2, 4, 47), Vel(0, 1, -6) }}");

    // print
    print_pos_vel(&scene);

    // despawn
    assert!(!scene.despawn(ent0), "was able to despawn entity#0 twice!");
    assert!(scene.despawn(ent1), "entity#1 despawn returned false!");

    assert!(!scene.contains(ent1), "entity#1 wasn't dispawned!");
    println!("despawned entity#1 {{ Pos(-1, -2, -3), Vel(5, -10, 18) }}");

    // print
    print_pos_vel(&scene);

    // make sure ent2 was moved correctly
    assert_eq!(scene.get::<Pos>(ent2), Some(&Pos(2, 4, 47)), "entity#2's Pos wasn't moved properly");
    assert_eq!(scene.get::<Vel>(ent2), Some(&Vel(0, 1, -6)), "entity#2's Vel wasn't moved properly");
}

#[test]
fn despawn_many()
{
    const N: u64 = 100_000;

    let mut scene = Scene::default();

    for _ in 0..N
    {
        scene.spawn((Pos(0, 0, 0), Vel(0, 0, 0)));
    }
    let chunks = scene
        .archetype::<(Vel, Pos)>()
        .expect("(Pos, Vel) archetype wasn't created!")
        .chunks()
        .len();
    
    println!("spawned {} entities over {} chunks", N, chunks);

    for id in 0..N
    {
        scene.despawn(unsafe { Entity::from_id(id) });
    }

    let chunks = scene
        .archetype::<(Vel, Pos)>()
        .expect("(Pos, Vel) archetype wasn't created!")
        .chunks()
        .len();

    println!("archetype has {} chunks left", chunks);
}