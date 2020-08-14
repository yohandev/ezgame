pub use ezgame_macros::*;

mod ent;    // entity
mod cmp;    // component
            // system

mod arch;   // archetype
mod scn;    // scene

pub use ent::*;
pub use cmp::*;

pub use arch::*;
pub use scn::*;