use std::collections::HashMap;

use bevy::ecs::prelude::*;

use crate::{Binding, DeadZone};


#[derive(Component, Debug,  Clone,  )]
pub struct GamepadOwner(pub i32);


#[derive(Component, Debug,  Clone,  )]
pub struct GamepadBindMode(pub bool);

#[derive(Component, Debug,  Clone,  )]
pub struct GamepadDeadZone(pub HashMap<Binding,DeadZone>);