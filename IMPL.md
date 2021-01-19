systems:
```rust
use ezgame::prelude::*;

struct MySystem { }

#[on_event(Update)]
fn struct_sys(&mut self, query: Query<(&Position, &mut Health)>)
{
    // ...
}

#[on_event(Update)]
fn fn_sys(query: Query<&Velocity, &mut Position>)
{
    // ...
}
```

underneath the hood of: `#[on_event(/* */)]`
    - writes to a project wide file(`.txt`, custom, it doesn't matter) the full path of the system function in question
    - engine code generator reads that file of registered systems(with macro) and adds them to the system builder