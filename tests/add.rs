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
fn add_component()
{
    let mut scene = Scene::default();

    let ent0 = scene.spawn((Pos(1, 2, 3), Vel(9, 8, 7)));

    println!("{} has {:?} and {:?}", ent0, scene.get::<Pos>(ent0), scene.get::<Vel>(ent0));
    println!("adding a Name(\"My Entity 0\") to {}...", ent0);

    scene.add(ent0, (Name("My Entity 0"),));

    println!("{} has {:?} and {:?} and {:?}", ent0, scene.get::<Pos>(ent0), scene.get::<Vel>(ent0), scene.get::<Name>(ent0));
    
    let chunk = scene
        .archetype::<(Pos, Vel)>()
        .unwrap()
        .chunks()
        .first()
        .unwrap();
    println!("(Pos, Vel) chunk should be empty...");
    println!("(Pos, Vel) chunk is now:\n    Pos: {:?}\n    Vel: {:?}", chunk.components::<Pos>(), chunk.components::<Vel>());
}

#[test]
fn add_many()
{
    const N: u64 = 100_000;

    let mut scene = Scene::default();

    for _ in 0..N
    {
        let e = scene.spawn((Pos(0, 0, 0), Vel(0, 0, 0)));

        assert!(scene.get::<Pos>(e).is_some());
        assert!(scene.get::<Vel>(e).is_some());
        assert!(scene.get::<Name>(e).is_none());

        scene.add(e, (Name("..."),));

        assert!(scene.get::<Pos>(e).is_some());
        assert!(scene.get::<Vel>(e).is_some());
        assert!(scene.get::<Name>(e).is_some());
    }

    println!("done adding... proceeding to re-checks");

    for id in 0..N
    {
        let e = unsafe { Entity::from_id(id) };

        assert!(scene.get::<Pos>(e).is_some());
        assert!(scene.get::<Vel>(e).is_some());
        assert!(scene.get::<Name>(e).is_some());
    }
}