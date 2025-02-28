use bevy::ecs::prelude::*;


#[derive(Component, Debug,  Clone,  )]
pub struct GamepadOwner(pub i32);


pub struct GamepadBindMode(pub bool);