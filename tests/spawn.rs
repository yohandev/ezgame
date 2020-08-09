use ezgame::*;

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
fn spawn_cmp()
{
    let mut scene = Scene::default();

    #[derive(Debug)]
    struct Pos(f32, f32, f32);
    #[derive(Debug)]
    struct Vel(f32, f32, f32);
    #[derive(Debug)]
    struct Name(&'static str);

    impl Component for Pos { }
    impl Component for Vel { }
    impl Component for Name { }

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