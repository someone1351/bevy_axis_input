
use std::collections::{ HashMap, HashSet};
// use std::fmt::Debug;
use bevy::ecs::system::Resource;
use bevy::prelude::Entity;
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
    // pub player_bindings : HashMap<PlayerId,HashMap<(M,BindingGroup),(f32,f32,f32)>>, //[player][mapping,bind_group]=(scale,primary_dead,modifier_dead)
    pub player_bindings : HashMap<i32,HashMap<(M,Vec<Binding>),(f32,f32,f32)>>, //[player][mapping,bindings]=(scale,primary_dead,modifier_dead)
    pub player_bindings_updated :bool,

    pub mapping_repeats : HashMap<M,f32>, //[mapping]=repeat //
    // pub device_dead_zones : HashMap<(Device,Binding),DeadZone>, //

    // pub(super) player_mappings : HashMap<i32, HashMap<M,MappingVal>>, //[player][mapping]=mapping_val
    // pub(super) player_primary_mappings : HashMap<i32, HashMap<Binding,HashSet<(M,BindingGroup)>>>, //[player][primary_binding][(mapping,binding_group)]
    // pub(super) player_modifier_mappings : HashMap<i32, HashMap<Binding,HashSet<(M,BindingGroup)>>>, //[player][modifier_binding][(mapping,binding_group)]


    // pub player_devices : HashMap<PlayerId,HashSet<Device>>, //[player]=devices //
    pub device_player : HashMap::<Device,i32>,
    pub bind_mode_excludes : HashSet<Binding>, //[binding] //

    // pub(super) bind_mode_enabled:bool,

    //
    pub bind_mode_devices:HashSet<Device>, //
    // pub(super) bind_mode_bindings:HashSet<(Device,Binding)>,
    // pub(super) bind_mode_chain:HashMap<Device,Vec<Binding>>,

    pub bind_mode_start_dead:f32, //
    pub bind_mode_end_dead:f32, //

    //

    pub(super) gamepad_devices:Vec<Option<(Entity,String,Option<u16>,Option<u16>)>>,
    pub(super) gamepad_device_entity_map:HashMap<Entity,usize>,
}

impl<M:Eq> Default for InputMap<M> {
    fn default() -> Self {
        Self {
            player_bindings: Default::default(),
            player_bindings_updated: Default::default(),
            mapping_repeats:HashMap::new(),
            // player_mappings : HashMap::new(),
            // player_primary_mappings : HashMap::new(),
            // player_modifier_mappings : HashMap::new(),
            // player_devices : HashMap::new(),
            device_player:HashMap::new(),
            // device_dead_zones : HashMap::new(),
            // bind_mode_enabled:false,
            // player_bind_mode_devices: HashMap::new(),
            bind_mode_devices: HashSet::new(),
            // player_bind_mode_bindings: HashMap::new(),
            // bind_mode_bindings: HashSet::new(),
            // bind_mode_chain:Default::default(),
            bind_mode_start_dead:0.4,
            bind_mode_end_dead:0.2,
            bind_mode_excludes:HashSet::new(),
            gamepad_devices:Vec::new(),
            gamepad_device_entity_map:HashMap::new(),
        }
    }
}

//for binding, if multiple keys pressed, then last key pressed is the primary, and when any of them are released the binding is finished

//need to clear binding_val.player_mapping_bind_groups when set_player_devices, set_player_mapping_bindings ??
