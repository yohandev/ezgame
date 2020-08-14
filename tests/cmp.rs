//! tests the component derive macros

use ezgame::*;

#[derive(Component)]
struct CmpA;

#[derive(Component)]
struct CmpB;

#[derive(Component)]
struct CmpC(i32, u32);

#[test]
fn assert_unique_id()
{
    let a = CmpA::ID;
    let b = CmpB::ID;
    let c = CmpC::ID;

    println!("a: {:?}, b: {:?}, c: {:?}", a, b, c);

    assert_ne!(a, b, "a~b IDs conflicted");
    assert_ne!(a, c, "a~c IDs conflicted");
    assert_ne!(c, b, "c~b IDs conflicted");
}

#[test]
fn print_meta()
{
    println!("a: {:?}", CmpA::META);
    println!("b: {:?}", CmpB::META);
    println!("c: {:?}", CmpC::META);
}