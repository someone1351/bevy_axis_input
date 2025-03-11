
use std::collections::{ HashMap, HashSet};
// use std::fmt::Debug;
use bevy::ecs::system::Resource;
// use bevy::prelude::Entity;
// use bevy::prelude::IntoSystem;

use super::values::*;

/*
* should set device dead zone by single vec2 for deadpos, and then a second vec2 for dead range?
** could automatically calculate it?
*** need a way to zero and check neg/pos range
*** need way to get and set those values

** but dead zone can be the new zero point, but also the max pos and neg vals
*** need to recalc axis value based on between those values, ie
** or let steam or external apps handle it?
*/

#[derive(Resource)]

pub struct InputMap<M:Eq> {
    pub owner_bindings : HashMap<i32,HashMap<(M,Vec<Binding>),(f32,f32,f32)>>, //[owner][mapping,bindings]=(scale,primary_dead,modifier_dead)
    pub owner_bindings_updated :bool,
    pub mapping_repeats : HashMap<M,f32>,

    // pub bind_mode_excludes : HashSet<Binding>,
    pub bind_mode_owner_includes : HashMap<i32,HashSet<Binding>>, //[owner][binding]
    pub bind_mode_owner_excludes : HashMap<i32,HashSet<Binding>>, //[owner][binding]
    pub bind_mode_start_dead:f32,
    pub bind_mode_end_dead:f32,

    pub kbm_bind_mode:bool,
    pub kbm_owner:i32,

    // // pub device_player : HashMap::<Device,i32>,
    // // pub bind_mode_devices:HashSet<Device>, //
    // pub(super) gamepad_devices:Vec<Option<(Entity,String,Option<u16>,Option<u16>)>>,
    // pub(super) gamepad_device_entity_map:HashMap<Entity,usize>,

    // pub owner_kbm_mapping_inverts : HashSet<(i32,M)>,
    // pub owner_gamepad_mapping_inverts : HashSet<(i32,M)>,
    // pub owner_device_mapping_inverts : HashSet<(i32,Device,M)>,

}

impl<M:Eq> Default for InputMap<M> {
    fn default() -> Self {
        Self {
            owner_bindings: Default::default(),
            owner_bindings_updated: Default::default(),
            mapping_repeats:Default::default(),
            bind_mode_start_dead:0.4,
            bind_mode_end_dead:0.2,
            // bind_mode_excludes:HashSet::new(),
            bind_mode_owner_includes:Default::default(),
            bind_mode_owner_excludes:Default::default(),
            kbm_bind_mode: false,
            kbm_owner: 0,
        }
    }
}

//for binding, if multiple keys pressed, then last key pressed is the primary, and when any of them are released the binding is finished

//need to clear binding_val.player_mapping_bind_groups when set_player_devices, set_player_mapping_bindings ??
