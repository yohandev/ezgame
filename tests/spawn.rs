use ezgame::*;

#[test]
fn spawn_main_thread()
{
    let scene = Scene::new();

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
fn spawn_multi_threaded()
{
    use std::sync::{ Arc, Mutex };
    use std::time::Duration;
    use std::thread;
    
    const THREADS: usize = 16;
    const SPAWN: usize = 32;

    println!("spawning {} entities from {} threads...", SPAWN, THREADS);

    let scene = Arc::new(Scene::new());

    let entities = Arc::new(Mutex::new(vec![]));

    // do work...
    for t in 0..THREADS
    {
        let scene = Arc::clone(&scene);
        let entities = Arc::clone(&entities);

        thread::spawn(move ||
        {
            let mut spawned = vec![];

            for _ in 0..SPAWN
            {
                let ent = scene.spawn(());
                spawned.push(ent);

                println!("spawned {} from thread#{}", ent, t);
            }

            entities
                .lock()
                .unwrap()
                .extend_from_slice(&spawned);
        });
    }

    // sleep on main thread while workers finish...
    thread::sleep(Duration::from_millis(30));

    // verify no duplicates
    println!("done. checking for duplicates...");

    let entities = entities
        .lock()
        .unwrap()
        .clone();
    
    for i in &entities
    {
        let mut j = 0;

        for k in &entities
        {
            if i == k { j += 1; }
        }

        if j > 1
        {
            eprintln!("detected duplicates! {}", i)
        }
    }
}