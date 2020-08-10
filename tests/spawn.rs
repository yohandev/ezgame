use ezgame::*;

#[derive(Debug, PartialEq)]
struct Pos(f32, f32, f32);
#[derive(Debug, PartialEq)]
struct Vel(f32, f32, f32);
#[derive(Debug, PartialEq)]
struct Name(&'static str);

impl Component for Pos { }
impl Component for Vel { }
impl Component for Name { }

#[test]
fn spawn_main_thread()
{
    let mut scene = Scene::default();

    let ent0 = scene.spawn(());
    let ent1 = scene.spawn(());
    let ent2 = scene.spawn(());
    let ent3 = scene.spawn(());
    let ent4 = scene.spawn(());

    println!("spawned:");
    println!("0: {}", ent0);
    println!("1: {}", ent1);
    println!("2: {}", ent2);
    println!("3: {}", ent3);
    println!("4: {}", ent4);
}

#[test]
fn spawn_many()
{
    const N: usize = 100_000;

    let mut scene = Scene::default();

    for _ in 0..N
    {
        scene.spawn((Pos(0.0, 0.0, 0.0), Vel(0.0, 0.0, 0.0)));
    }
    let chunks = scene
        .archetype::<(Vel, Pos)>()
        .expect("(Pos, Vel) archetype wasn't created!")
        .chunks()
        .len();
    
    println!("spawned {} entities over {} chunks", N, chunks);
}

#[test]
fn spawn_cmp()
{
    let mut scene = Scene::default();

    let _ = scene.spawn(());
    let _ = scene.spawn((Pos(0.0, 1.0, 27.0), Vel(0.0, -9.8, 0.0)));
    let _ = scene.spawn((Pos(1.0, -5.0, -2.0), Vel(10.0, 1.2, 5.3)));
    let _ = scene.spawn((Pos(10.0, 10.0, 10.0), Vel(5.0, 5.0, 5.0), Name("Entity#3")));

    let pos_vel = scene
        .archetype::<(Pos, Vel)>()
        .expect("(Pos, Vel) archetype wasn't created!");
    let pos_vel_chunk = pos_vel
        .chunks()
        .first()
        .expect("(Pos, Vel) archetype has no chunks!");

    println!("(Pos, Vel) -> Ent: {:?}", pos_vel_chunk.entities());
    println!("(Pos, Vel) -> Pos: {:?}", pos_vel_chunk.components::<Pos>());
    println!("(Pos, Vel) -> Vel: {:?}", pos_vel_chunk.components::<Vel>());

    let pos_vel_name = scene
        .archetype::<(Pos, Vel, Name)>()
        .expect("(Pos, Vel, Name) archetype wasn't created!");
    let pos_vel_name_chunk = pos_vel_name
        .chunks()
        .first()
        .expect("(Pos, Vel, Name) archetype has no chunks!");

    println!("(Pos, Vel, Name) -> Ent: {:?}", pos_vel_name_chunk.entities());
    println!("(Pos, Vel, Name) -> Pos: {:?}", pos_vel_name_chunk.components::<Pos>());
    println!("(Pos, Vel, Name) -> Vel: {:?}", pos_vel_name_chunk.components::<Vel>());
    println!("(Pos, Vel, Name) -> Name: {:?}", pos_vel_name_chunk.components::<Name>());
}

// note: this test might fail on some platforms due to floating point (in)equality
#[test]
fn spawn_then_get()
{
    let mut scene = Scene::default();

    let ent0 = scene.spawn(());
    let ent1 = scene.spawn((Pos(0.0, 1.0, 27.0), Vel(0.0, -9.8, 0.0)));
    let ent2 = scene.spawn((Pos(1.0, -5.0, -2.0), Vel(10.0, 1.2, 5.3)));
    let ent3 = scene.spawn((Pos(10.0, 10.0, 10.0), Vel(5.0, 5.0, 5.0), Name("Entity#3")));

    assert_eq!(scene.get::<Pos>(ent0), None);
    assert_eq!(scene.get::<Vel>(ent0), None);
    assert_eq!(scene.get::<Name>(ent0), None);

    assert_eq!(scene.get::<Pos>(ent1), Some(&Pos(0.0, 1.0, 27.0)));
    assert_eq!(scene.get::<Vel>(ent1), Some(&Vel(0.0, -9.8, 0.0)));
    assert_eq!(scene.get::<Name>(ent1), None);

    assert_eq!(scene.get::<Pos>(ent2), Some(&Pos(1.0, -5.0, -2.0)));
    assert_eq!(scene.get::<Vel>(ent2), Some(&Vel(10.0, 1.2, 5.3)));
    assert_eq!(scene.get::<Name>(ent2), None);

    assert_eq!(scene.get::<Pos>(ent3), Some(&Pos(10.0, 10.0, 10.0)));
    assert_eq!(scene.get::<Vel>(ent3), Some(&Vel(5.0, 5.0, 5.0)));
    assert_eq!(scene.get::<Name>(ent3), Some(&Name("Entity#3")));

    // mutably borrow component(new scope)
    {
        let pos1 = scene
            .get_mut::<Pos>(ent1)
            .expect("can't get Pos mutably for ent1");

        pos1.0 = 600.0;
        pos1.2 = 500.0;
    }
    assert_eq!(scene.get::<Pos>(ent1), Some(&Pos(600.0, 1.0, 500.0)));

    assert_eq!(scene.get::<Pos>(ent0), None);
    assert_eq!(scene.get::<Pos>(ent2), Some(&Pos(1.0, -5.0, -2.0)));
    assert_eq!(scene.get::<Pos>(ent3), Some(&Pos(10.0, 10.0, 10.0)));
}

// #[test]
// fn spawn_multi_threaded()
// {
//     use std::sync::{ Arc, Mutex };
//     use std::time::Duration;
//     use std::thread;
    
//     const THREADS: usize = 16;
//     const SPAWN: usize = 32;

//     println!("spawning {} entities from {} threads...", SPAWN, THREADS);

//     let scene = Arc::new(Scene::default());

//     let entities = Arc::new(Mutex::new(vec![]));

//     // do work...
//     for t in 0..THREADS
//     {
//         let scene = Arc::clone(&scene);
//         let entities = Arc::clone(&entities);

//         thread::spawn(move ||
//         {
//             let mut spawned = vec![];

//             for _ in 0..SPAWN
//             {
//                 let ent = scene.spawn(());
//                 spawned.push(ent);

//                 println!("spawned {} from thread#{}", ent, t);
//             }

//             entities
//                 .lock()
//                 .unwrap()
//                 .extend_from_slice(&spawned);
//         });
//     }

//     // sleep on main thread while workers finish...
//     thread::sleep(Duration::from_millis(30));

//     // verify no duplicates
//     println!("done. checking for duplicates...");

//     let entities = entities
//         .lock()
//         .unwrap()
//         .clone();
    
//     for i in &entities
//     {
//         let mut j = 0;

//         for k in &entities
//         {
//             if i == k { j += 1; }
//         }

//         if j > 1
//         {
//             eprintln!("detected duplicates! {}", i)
//         }
//     }
// }