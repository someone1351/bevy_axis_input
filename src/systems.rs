/*
* slight problem with bind mode, on multiple keys
** sometimes if one key released and then another pressed, will count as both pressed
*** could check when one is released, check if the other(s) were just pressed, and if so ignore them

* if same bindgroup bound to multiple mappings, then when pressed, only one of the mappings will receive the input, is that a problem?
** makes sense that the more modifiers a bindgroup has, that it will be used for input, but what about exact same?

* if for example ctrl+s, is bound, and is pressed, but the modifier is released (not the primary), the value will equal -0.0 instead of just 0.0, why?

*/
use std::{collections::{HashMap, HashSet}, fmt::Debug, hash::Hash};

use bevy::{ecs::prelude::*, prelude::{Gamepad, GamepadAxis}};
use bevy::input::gamepad::{GamepadAxisChangedEvent, GamepadButtonChangedEvent, GamepadConnection, GamepadConnectionEvent, GamepadEvent,};
use bevy::input::keyboard::KeyCode;

use crate::{GamepadBindMode, GamepadDeadZone, GamepadOwner};

use super::resources::*;
use super::events::*;
use super::values::*;

fn use_dead_zone(value:f32,dead_zone:Option<&DeadZone>) -> f32 {
    let Some(dead_zone)=dead_zone else {
        return value;
    };

    let pos_min=dead_zone.pos_min.max(dead_zone.neg_min);
    let neg_min=dead_zone.neg_min.min(dead_zone.pos_min);
    let pos_max=dead_zone.pos_max.max(pos_min);
    let neg_max=dead_zone.neg_max.min(neg_min);

    if value > pos_min {
        let len=pos_max-pos_min;

        if len>0.0 {
            return value.clamp(pos_min,pos_max)/len;
        }

    } else if value < neg_min {
        let len=neg_max-neg_min;

        if len>0.0 {
            return value.clamp(neg_max,neg_min)/len;
        }
    }

    0.0
}

fn is_binding_bind_mode(
    // bind_mode : bool,
    // owner : Option<i32>,
    // owner : i32,
    // owner_excludes : &HashMap<i32,HashSet<Binding>>,
    // owner_includes : &HashMap<i32,HashSet<Binding>>,
    bind_mode_excludes : &HashSet<Binding>,
    bind_mode_includes : &HashSet<Binding>,
    binding : Binding,
) -> bool {
    //
    // if !bind_mode {
    //     return false;
    // }

    //
    // if let Some(bind_mode_excludes)=owner_excludes.get(&owner) {
        if bind_mode_excludes.contains(&binding) {
            return false;
        }
    // }

    //
    // if let Some(bind_mode_includes)=owner_includes.get(&owner) {
        if !bind_mode_includes.is_empty() && bind_mode_includes.contains(&binding) {
            return false;
        }
    // }

    //
    true
}

pub fn binding_inputs_system<M: Send + Sync + 'static + Eq + Debug> (
    mut gamepad_events: EventReader<GamepadEvent>,
    mut key_events: EventReader<bevy::input::keyboard::KeyboardInput>,
    mut mouse_move_events: EventReader<bevy::input::mouse::MouseMotion>,
    mut mouse_scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut mouse_button_events : EventReader<bevy::input::mouse::MouseButtonInput>,

    mut gamepad_axis_lasts : Local<HashMap<(Device,GamepadAxis),f32>>,
    mut key_lasts : Local<HashSet<KeyCode>>,

    mut binding_input_event_writer: EventWriter<BindingInputEvent>,

    gamepad_dead_zones_query: Query<& GamepadDeadZone>,
) {
    //
    for event in gamepad_events.read() {
        let immediate=false; //what's this for? for differentiating mouse move from the rest?

        match event {
            GamepadEvent::Connection(..) => { }
            GamepadEvent::Button(GamepadButtonChangedEvent {value, entity, button:button_type, .. })=> {
                let entity=*entity;
                let device=Device::Gamepad(entity);
                let binding=Binding::GamepadButton(*button_type);
                let dead_zone=gamepad_dead_zones_query.get(entity).ok().and_then(|dead_zones|dead_zones.0.get(&binding));
                let value=use_dead_zone(*value,dead_zone);

                binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
            }
            GamepadEvent::Axis(GamepadAxisChangedEvent {value, entity, axis:axis_type })=> {
                let entity=*entity;
                let axis_type=*axis_type;
                let device=Device::Gamepad(entity);
                let binding=Binding::GamepadAxis(axis_type);
                let dead_zone=gamepad_dead_zones_query.get(entity).ok().and_then(|dead_zones|dead_zones.0.get(&binding));
                let value=use_dead_zone(*value,dead_zone);
                let last_value=gamepad_axis_lasts.get(&(device,axis_type)).cloned().unwrap_or_default();

                binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });

                //the "or" part is so to know if last val had been pos and cur val is <=0, so knows to send an event with val=0
                if value>0.0 || last_value>0.0 && value <= 0.0 {
                    let value=value.max(0.0);
                    let binding=Binding::GamepadAxisPos(axis_type);
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }

                if value<0.0 || last_value<0.0 && value >= 0.0 {
                    let value=value.min(0.0).abs();
                    let binding=Binding::GamepadAxisNeg(axis_type);
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }

                //
                gamepad_axis_lasts.insert((device,axis_type), value);
            }
        }
    }

    //
    for ev in key_events.read() {
        let immediate=false;

        match ev.state {
            bevy::input::ButtonState::Pressed => { //repeats
                if !key_lasts.contains(&ev.key_code) { //don't send if just a repeat
                    let device=Device::Other;
                    let binding=Binding::Key(ev.key_code);
                    let value=1.0;
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                    key_lasts.insert(ev.key_code);
                }
            }
            bevy::input::ButtonState::Released => { //once
                let device=Device::Other;
                let value=0.0;
                let binding=Binding::Key(ev.key_code);
                binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                key_lasts.remove(&ev.key_code); //may not exist, if there was somehow a release without a press
            }
        }
    }

    //
    for ev in mouse_button_events.read() {
        let immediate=false;

        match ev.state {
            bevy::input::ButtonState::Pressed => {
                let device=Device::Other;
                let binding=Binding::MouseButton(ev.button);
                let value=1.0;
                binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
            }
            bevy::input::ButtonState::Released => {
                let device=Device::Other;
                let binding=Binding::MouseButton(ev.button);
                let value=0.0;
                binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
            }
        }
    }

    //
    for ev in mouse_move_events.read() {
        let immediate=true;
        let device=Device::Other;

        if ev.delta.x!=0.0 {
            let binding=Binding::MouseMoveX;
            let value=ev.delta.x;
            binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
        }
        if ev.delta.x>0.0 {
            let binding=Binding::MouseMovePosX;
            let value=ev.delta.x;
            binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
        }
        if ev.delta.x<0.0 {
            let binding=Binding::MouseMoveNegX;
            let value=ev.delta.x;
            binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
        }

        if ev.delta.y!=0.0 {
            let binding=Binding::MouseMoveY;
            let value=ev.delta.y;
            binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
        }
        if ev.delta.y>0.0 {
            let binding=Binding::MouseMovePosY;
            let value=ev.delta.y;
            binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
        }
        if ev.delta.y<0.0 {
            let binding=Binding::MouseMoveNegY;
            let value=ev.delta.y;
            binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
        }
    }

    //
    for ev in mouse_scroll_events.read() {
        let immediate=true;
        let device=Device::Other;

        match ev.unit {
            bevy::input::mouse::MouseScrollUnit::Line => {
                if ev.x!=0.0 {
                    let binding=Binding::MouseScrollLineX;
                    let value=ev.x;
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }
                if ev.x>0.0 {
                    let binding=Binding::MouseScrollLinePosX;
                    let value=ev.x;
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }
                if ev.x<0.0 {
                    let binding=Binding::MouseScrollLineNegX;
                    let value=ev.x;
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }

                if ev.y!=0.0 {
                    let binding=Binding::MouseScrollLineY;
                    let value=ev.y;
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }
                if ev.y>0.0 {
                    let binding=Binding::MouseScrollLinePosY;
                    let value=ev.y;
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }
                if ev.y<0.0 {
                    let binding=Binding::MouseScrollLineNegY;
                    let value=ev.y;
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }
            }
            bevy::input::mouse::MouseScrollUnit::Pixel => {
                // println!("!==w3erfdsfdsfds");
                if ev.x!=0.0 {
                    let binding=Binding::MouseScrollPixelX;
                    let value=ev.x;
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }
                if ev.x>0.0 {
                    let binding=Binding::MouseScrollPixelPosX;
                    let value=ev.x;
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }
                if ev.x<0.0 {
                    let binding=Binding::MouseScrollPixelNegX;
                    let value=ev.x;
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }

                if ev.y!=0.0 {
                    let binding=Binding::MouseScrollPixelY;
                    let value=ev.y;
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }
                if ev.y>0.0 {
                    let binding=Binding::MouseScrollPixelPosY;
                    let value=ev.y;
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }
                if ev.y<0.0 {
                    let binding=Binding::MouseScrollPixelNegY;
                    let value=ev.y;
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }
            }
        }
    }

    //
}



pub fn mapping_event_system<M: Send + Sync + 'static + Eq + Hash+Clone+core::fmt::Debug> (
    mut gamepad_events: EventReader<GamepadEvent>,
    mut binding_input_events: EventReader<BindingInputEvent>,
    mut mapping_event_writer: EventWriter<InputMapEvent<M>>,

    mut input_map : ResMut<InputMap<M>>,
    time: Res<bevy::time::Time>,

    gamepad_query: Query<(Entity,Option<& GamepadOwner>,Option<&GamepadBindMode>),With<Gamepad>>,
    mut device_prev_owners : Local<HashMap<Device,i32>>,

    mut bind_mode_bindings:Local<HashSet<(Device,Binding)>>,
    mut bind_mode_chain:Local<HashMap<Device,Vec<Binding>>>,

    mut device_bind_mode_lasts : Local<HashSet<Device>>,

    mut modifier_binding_vals : Local<HashMap<(Device,Binding),f32>>, //not just modifier, all binding vals except for immediate ones, zero val are not stored

    mut owner_mappings : Local<HashMap<i32, HashMap<M,MappingVal>>>, //[player][mapping]=mapping_val
    mut owner_primary_mappings : Local<HashMap<i32, HashMap<Binding,HashSet<(M,BindingGroup)>>>>, //[player][primary_binding][(mapping,binding_group)]
    mut owner_modifier_mappings : Local<HashMap<i32, HashMap<Binding,HashSet<(M,BindingGroup)>>>>, //[player][modifier_binding][(mapping,binding_group)]


    mut other_device_owners : Local<HashSet<i32>>,
) {
    let InputMap {
        owner_bindings, bindings_updated: owner_bindings_updated,
        mapping_repeats,
        // bind_mode_owner_includes,
        // bind_mode_owner_excludes,
        bind_mode_includes,
        bind_mode_excludes,
        bind_mode_start_dead,bind_mode_end_dead,
        // kbm_owner,
        kbm_bind_mode: bind_mode_kbm,
        ..
    }=input_map.as_mut();

    //store last mapping bindings sum val
    //slightly sucks to calculate this all the time,
    //  would prefer to do when needed for removes,
    //  or run this system as fixed time step
    let mut owner_mapping_last_vals : HashMap<(i32,M),f32> = HashMap::new(); //[owner,mapping]=last_val

    for (&owner, mapping_vals) in owner_mappings.iter() {
        for (mapping,mapping_val) in mapping_vals.iter() {
            let last_val = owner_mapping_last_vals.entry((owner,mapping.clone())).or_default();
            *last_val=mapping_val.binding_vals.iter().map(|(_,&v)|v).sum::<f32>();
        }
    }

    //
    let mut device_owner = HashMap::new();
    // device_owner.insert(Device::Other, *kbm_owner);

    for (entity,owner,_) in gamepad_query.iter() {
        let Some(owner)=owner.map(|x|x.0) else {continue;};
        let device=Device::Gamepad(entity);
        device_owner.insert(device,owner);
    }

    //calc bind_mode_devices
    let mut bind_mode_devices:HashSet<Device> = HashSet::new();

    if *bind_mode_kbm {
        bind_mode_devices.insert(Device::Other);
    }

    for (entity,_,bind_mode) in gamepad_query.iter() {
        if let Some(bind_mode)=bind_mode {
            if bind_mode.0 {
                bind_mode_devices.insert(Device::Gamepad(entity));
            }
        }
    }

    //
    // let mut owner_mapping_changeds  = HashSet::<(i32,M)>::new(); //[(owner,mapping]=changed
    let mut owner_mapping_changeds  = HashSet::<(i32,Option<M>)>::new(); //[(owner,mapping]=changed
    // let mut device_removeds  = HashMap::<(Device, i32),bool>::new(); //[device,owner]=bind_mode_only

    //handle bind_mode removes
    //  clear presseds on bind mode
    for &device in bind_mode_devices.iter() {
        if !device_bind_mode_lasts.contains(&device) {
            let owners=if let Device::Other=device {
                other_device_owners.iter().cloned().collect::<Vec<_>>()
            } else if let Some(&owner)=device_owner.get(&device) {
                vec![owner]
            } else {
                Vec::new()
            };

            // let Some(&owner)=device_owner.get(&device) else {continue;};
            // //check last owner is same, otherwise pointless?
            // // device_removeds.insert((device,owner),true);

            // // let bind_mode_excludes=bind_mode_owner_excludes.get(&owner);
            // // let bind_mode_includes=bind_mode_owner_includes.get(&owner);


            for owner in owners {
                owner_mapping_changeds.insert((owner,None));

                //
                let Some(mapping_vals)=owner_mappings.get_mut(&owner) else { continue; };


                //clear presseds on bind mode

                for (_mapping,mapping_val) in mapping_vals.iter_mut() {
                    //remove bind_groups that have device in bindmode, and aren't excluded from it
                    mapping_val.binding_vals.retain(|(device2,bind_group),_|{
                        let not_bind_mode=!bind_mode_devices.contains(device2);

                        // let not_bind_mode=not_bind_mode || bind_mode_excludes.contains(&bind_group.primary);
                        // let not_bind_mode=not_bind_mode || !is_binding_bind_mode(owner,&bind_mode_owner_excludes,&bind_mode_owner_includes,bind_group.primary);
                        let not_bind_mode=not_bind_mode || !is_binding_bind_mode(&bind_mode_excludes,&bind_mode_includes,bind_group.primary);

                        // let not_bind_mode=not_bind_mode || bind_group.modifiers.len()==bind_group.modifiers.iter().filter(|&&x|bind_mode_excludes.contains(x)).count();
                        let not_bind_mode=not_bind_mode || bind_group.modifiers.len()==bind_group.modifiers.iter().filter(|&&binding|{
                            // !is_binding_bind_mode(owner,&bind_mode_owner_excludes,&bind_mode_owner_includes,binding)
                            !is_binding_bind_mode(&bind_mode_excludes,&bind_mode_includes,binding)
                        }).count();

                        not_bind_mode
                    });
                }
            }

        }
    }

    //need to handle kbm that has player removed from it
    // {
    //     let device=Device::Other;
    //     let last_owner=device_prev_owners.get(&device).cloned().unwrap_or(0);
    //     let cur_owner=*kbm_owner;

    //     if last_owner!=cur_owner {
    //         device_prev_owners.insert(device, cur_owner);
    //         //check last owner is same, otherwise pointless?
    //         // device_removeds.insert((device,last_owner),false);
    //         owner_mapping_changeds.insert((last_owner,None));
    //     }
    // }

    //need to handle gamepad that has player removed from it
    for (entity,owner,_bind_mode) in gamepad_query.iter() {
        let device=Device::Gamepad(entity);
        let last_owner=device_prev_owners.get(&device).cloned();
        let cur_owner=owner.map(|x|x.0);

        if last_owner!=cur_owner {
            if let Some(cur_owner)=cur_owner {
                device_prev_owners.insert(device, cur_owner);
            } else {
                device_prev_owners.remove(&device).unwrap();
            }

            if let Some(last_owner)=last_owner {
                // device_removeds.insert((device,last_owner),false);
                owner_mapping_changeds.insert((last_owner,None));
            }
        }
    }

    //
    let mut disconnected_devices = HashSet::<Device>::new();

    //release inputs of disconnected gamepad
    for event in gamepad_events.read() {
        let GamepadEvent::Connection(GamepadConnectionEvent{gamepad,connection:GamepadConnection::Disconnected})=event else {continue;};
        let gamepad_device=Device::Gamepad(*gamepad);
        let Some(owner)=device_owner.get(&gamepad_device).cloned() else {continue;};

        //check last owner is same, otherwise pointless?
        // device_removeds.insert((gamepad_device,owner),false);

        disconnected_devices.insert(gamepad_device);
        owner_mapping_changeds.insert((owner,None));owner_mapping_changeds.insert((owner,None));
        // let Some(mapping_vals)=owner_mappings.get(&owner) else { continue; };

        // for mapping in mapping_vals.keys() {
        //     owner_mapping_changeds.insert((owner,mapping.clone()));
        // }
    }

    //clear modifier_binding_vals with disconnected devices
    if !disconnected_devices.is_empty() {
        modifier_binding_vals.retain(|(device,_),_|!disconnected_devices.contains(device));
    }

    //
    let mut not_repeatings : HashSet<(i32, M)> = Default::default();


    //on mappings/bindings updated
    //send events for removed mappings ending? also bindings?
    if *owner_bindings_updated {
        *owner_bindings_updated=false;
        other_device_owners.clear();

        for (&owner,mappings) in owner_bindings.iter() {
            let mut temp_owner_mappings: HashMap<M, HashMap<BindingGroup,MappingBindingInfo>>=HashMap::new();

            //collect input in temp mappings
            for ((mapping, bindings),&(scale, primary_dead, modifier_dead)) in mappings.iter() {
                if bindings.is_empty() {
                    continue;
                }

                for binding in bindings.iter() {
                    if binding.is_other_device() {
                        other_device_owners.insert(owner);
                        break;
                    }
                }

                let temp_bindings=temp_owner_mappings.entry(mapping.clone()).or_default();
                let binding_group=BindingGroup{ modifiers: bindings[0..bindings.len()-1].to_vec(), primary: bindings.last().unwrap().clone() };

                temp_bindings.insert(binding_group,MappingBindingInfo{scale,primary_dead,modifier_dead}); //,binding_val:0.0
            }

            //setup primary binding mappings
            {
                let primary_mappings=owner_primary_mappings.entry(owner).or_default();
                primary_mappings.clear();

                for (mapping,temp_bindings) in temp_owner_mappings.iter() {
                    for (bind_group,_) in temp_bindings.iter() {
                        primary_mappings.entry(bind_group.primary).or_default().insert((mapping.clone(),bind_group.clone()));
                    }
                }
            }

            //setup modifier binding mappings
            {
                let modifier_mappings=owner_modifier_mappings.entry(owner).or_default();
                modifier_mappings.clear();

                for (mapping,temp_bindings) in temp_owner_mappings.iter() {
                    for (bind_group,_) in temp_bindings.iter() {
                        for &modifier in bind_group.modifiers.iter() {
                            modifier_mappings.entry(modifier).or_default().insert((mapping.clone(),bind_group.clone()));
                        }
                    }
                }
            }

            //setup/insert mappings
            {
                let mappings=owner_mappings.entry(owner).or_insert_with(Default::default);

                //remove mappings from player_mappings not in temp
                let removed_mappings=mappings.iter().filter_map(|(k,_)|(!temp_owner_mappings.contains_key(k)).then_some(k.clone())).collect::<Vec<_>>();
                // let removed_mappings=mappings.drain_filter(|k,_|temp_owner_mappings.contains_key(k)).collect::<Vec<_>>();

                for mapping in removed_mappings {

                    owner_mapping_changeds.insert((owner,Some(mapping.clone())));

                }

                mappings.retain(|k,_|temp_owner_mappings.contains_key(k));

                //add new mappings/binding_infos or replace bindings in player_mapping from temp
                for (mapping,temp_bindings) in temp_owner_mappings {
                    let mapping_val=mappings.entry(mapping.clone()).or_default();

                    //remove any cur binding valss that are no longer used
                    for k in mapping_val.binding_vals.keys().cloned().collect::<Vec<_>>() {
                        if !temp_bindings.contains_key(&k.1) {
                            mapping_val.binding_vals.remove(&k).unwrap();

                            // owner_mapping_changeds.insert((owner,mapping.clone()));
                            owner_mapping_changeds.insert((owner,None));
                        }
                    }

                    //
                    mapping_val.binding_infos=temp_bindings;
                }
            }
        }
    }

    //
    let binding_inputs=binding_input_events.read().map(|&x|x).collect::<Vec<_>>();

    //get (inputs for) modifier presses/releases
    for binding_input in binding_inputs.iter() {
        if binding_input.immediate {
            continue;
        }

        let device_binding=(binding_input.device,binding_input.binding);

        // let owner = device_owner.get(&binding_input.device).cloned();
        // let bind_mode_excludes=owner.and_then(|owner|bind_mode_owner_excludes.get(&owner));

        //should modifier_binding_vals be renamed to binding_vals? but only used for modifiers ..., also immediate values not stored
        let is_bind_mode=bind_mode_devices.contains(&binding_input.device);
        // let bind_mode=bind_mode&&!bind_mode_excludes.contains(&binding_input.binding);
        // let is_bind_mode=is_bind_mode&&owner.map(|owner|is_binding_bind_mode(owner,&bind_mode_owner_excludes,&bind_mode_owner_includes,binding_input.binding)).unwrap_or(false);
        let is_bind_mode=is_bind_mode&&is_binding_bind_mode(&bind_mode_excludes,&bind_mode_includes,binding_input.binding);

        if is_bind_mode || binding_input.value == 0.0 {
            modifier_binding_vals.remove(&device_binding);
        } else {
            modifier_binding_vals.insert(device_binding,binding_input.value);
        }
    }

    //on binding release, check all pressed binding_groups, that use that modifier and remove/depress them
    //  need to do all at once
    //  need to check if binding_input's device is in bind_mode? to ignore? no since already removed above?
    for binding_input in binding_inputs.iter() {
        //
        if binding_input.value!=0.0 || binding_input.immediate {
            continue;
        }

        //
        // let Some(owner)=device_owner.get(&binding_input.device).cloned() else { continue; };

        //
        let owners=if let Device::Other=binding_input.device {
            other_device_owners.iter().cloned().collect::<Vec<_>>()
        } else if let Some(&owner)=device_owner.get(&binding_input.device) {
            vec![owner]
        } else {
            Vec::new()
        };

        for owner in owners {
            //
            let Some(modifier_mappings) = owner_modifier_mappings.get(&owner).and_then(|modifier_mappings|modifier_mappings.get(&binding_input.binding)) else {
                continue;
            };

            //
            let Some(mapping_vals) = owner_mappings.get_mut(&owner) else { continue; };

            //find (mapping,device,bind_groups) that released input binding is a modifier of
            //  use removeds for sending events
            for (mapping,bind_group) in modifier_mappings.iter() {
                //get mapping val
                let mapping_val = mapping_vals.get_mut(mapping).unwrap();

                //get/init binding_vals
                // let binding_vals=owner_mapping_binding_vals.entry((owner,mapping.clone())).or_insert_with(||mapping_val.binding_vals.clone());

                //
                let device_bind_group=(binding_input.device,bind_group.clone());
                let binding_val=mapping_val.binding_vals.get(&device_bind_group).cloned().unwrap_or_default();

                if binding_val!=0.0 {

                    // owner_mapping_changeds.insert((owner,mapping.clone()));

                    owner_mapping_changeds.insert((owner,None));

                    // device_removeds.insert((binding_input.device,owner), false);
                    //todo need to do release, valuechange, and press (if necessary)
                    mapping_val.binding_vals.remove(&device_bind_group).unwrap();

                }
            }
        }
    }

    //handle mapping/binding/owner/connection changes
    for (owner,mapping) in owner_mapping_changeds {
        if let Some(mapping)=mapping { //removed mappings
            let owner_mapping=(owner,mapping.clone());

            let last_val = *owner_mapping_last_vals.get(&owner_mapping).unwrap();
            let last_dir=if last_val>0.0{1}else if last_val<0.0{-1}else{0};

            if last_val!=0.0 {
                mapping_event_writer.send(InputMapEvent::ValueChanged { mapping: mapping.clone(), val: 0.0, owner });
            }

            if last_val==0.0 {
                mapping_event_writer.send(InputMapEvent::JustReleased { mapping: mapping.clone(), dir: last_dir, owner });
            }
        } else {
            let Some(mapping_vals)=owner_mappings.get(&owner) else {
                //possible an owner with mappings removed is added to owner_mapping_changeds
                //  due owner mappings being removed, while device owner being removed or device being disconnected
                continue;
            };

            //recalc mapping's binding val sum, and check if anything has changed, ie binding removed, therefore mapping no longer pressed
            for (mapping,mapping_val) in mapping_vals.iter() {
                let owner_mapping=(owner,mapping.clone());

                let last_val = *owner_mapping_last_vals.get(&owner_mapping).unwrap();
                let last_dir=if last_val>0.0{1}else if last_val<0.0{-1}else{0};

                let cur_val=mapping_val.binding_vals.iter().map(|(_,&v)|v).sum::<f32>();
                let cur_dir=if cur_val>0.0{1}else if cur_val<0.0{-1}else{0};

                if last_val!=cur_val {
                    mapping_event_writer.send(InputMapEvent::ValueChanged { mapping: mapping.clone(), val: cur_val, owner });
                }

                if cur_dir!=last_dir {
                    mapping_event_writer.send(InputMapEvent::JustReleased { mapping: mapping.clone(), dir: last_dir, owner });

                    if cur_val!=0.0 {
                        mapping_event_writer.send(InputMapEvent::JustPressed { mapping: mapping.clone(), dir: cur_dir, owner });
                    }
                }
            }
        }
    }

    //handle primary presses, primary depresses
    //  maybe should handle modifier presses/depresses here?
    //     probably not since modifiers being pressed is handled above
    //     though kinda makes sense
    //     although reason not to is because kb and ms are considered the same device
    //       but their inputs are received separately
    //       so if say chose to receive kb inputs first and ms second
    //         then could never use ms inputs as modifiers and kb input as primary
    for binding_input in binding_inputs.iter() {
        //
        // let Some(owner)=device_owner.get(&binding_input.device).cloned() else { continue; };

        //
        let owners=if let Device::Other=binding_input.device {
            other_device_owners.iter().cloned().collect::<Vec<_>>()
        } else if let Some(&owner)=device_owner.get(&binding_input.device) {
            vec![owner]
        } else {
            Vec::new()
        };

        for owner in owners {
            let is_bind_mode=bind_mode_devices.contains(&binding_input.device);
            // let bind_mode_excludes=bind_mode_owner_excludes.get(&owner);
            // let is_bind_mode=is_bind_mode && !bind_mode_excludes.contains(&binding_input.binding);
            // let is_bind_mode=is_bind_mode && is_binding_bind_mode(owner,&bind_mode_owner_excludes,&bind_mode_owner_includes,binding_input.binding);
            let is_bind_mode=is_bind_mode && is_binding_bind_mode(&bind_mode_excludes,&bind_mode_includes,binding_input.binding);

            if is_bind_mode {
                continue;
            }

            //
            let Some(mapping_vals) = owner_mappings.get_mut(&owner) else { continue; };

            //get (mapping,binding_group)'s, with binding_input.binding as primary
            let Some(primary_mapping_binding_group_set) = owner_primary_mappings.get(&owner)
                .and_then(|binding_mappings|binding_mappings.get(&binding_input.binding)) else
            {
                continue;
            };

            //
            let mut founds: Vec<(M, BindingGroup,)> = Vec::new();

            for (mapping,bind_group) in primary_mapping_binding_group_set.iter() {
                let mapping_val = mapping_vals.get(mapping).unwrap();
                let binding_val=mapping_val.binding_vals.get(&(binding_input.device,bind_group.clone())).cloned().unwrap_or_default();

                if binding_val!=0.0 {
                    // let mapping_val = mapping_vals.get(mapping).unwrap();
                    // let binding_info=mapping_val.binding_infos.get(bind_group).unwrap();

                    //not needed since mapping binding_groups that modifiers not pressed are removed above
                    // let modifier_pressed_count=bind_group.modifiers.iter().filter(|&&modifier_bind|{
                    //     let modifier_val=modifier_binding_vals.get(&(binding_input.device,modifier_bind)).cloned().unwrap_or_default();
                    //     let modifier_val = if modifier_val.abs()<binding_info.modifier_dead{0.0}else{modifier_val};
                    //     modifier_val==0.0
                    // }).count();

                    // let modifiers_pressed=modifier_pressed_count==bind_group.modifiers.len();

                    founds.push((mapping.clone(),bind_group.clone(),));
                }
            }

            if founds.is_empty() {

                // let bind_mode_excludes=bind_mode_owner_excludes.get(&owner);

                //get valid binding mappings
                let mut primary_mapping_binding_group_vec=primary_mapping_binding_group_set.iter()
                    .map(|x|x.clone()).collect::<Vec<_>>();
                primary_mapping_binding_group_vec.sort_by(|a,b|b.1.modifiers.len().cmp(&a.1.modifiers.len()));

                //keep ones with all modifiers (if any) pressed
                primary_mapping_binding_group_vec.retain(|(mapping,bind_group)|{
                    let mapping_val = mapping_vals.get(mapping).unwrap();
                    let binding_info=mapping_val.binding_infos.get(bind_group).unwrap();

                    //check modifiers pressed
                    for &modifier_binding in bind_group.modifiers.iter() {
                        let modifier_val=modifier_binding_vals.get(&(binding_input.device,modifier_binding)).cloned().unwrap_or_default();
                        let modifier_val = if modifier_val.abs()<binding_info.modifier_dead{0.0}else{modifier_val};

                        if modifier_val== 0.0 || (is_bind_mode &&
                            // !bind_mode_excludes.contains(&modifier_binding)
                            // is_binding_bind_mode(owner,&bind_mode_owner_excludes,&bind_mode_owner_includes,modifier_binding)
                            is_binding_bind_mode(&bind_mode_excludes,&bind_mode_includes,modifier_binding)
                        ) {
                            return false;
                        }
                    }

                    //
                    true
                });


                if let Some((_mapping,binding_group))=primary_mapping_binding_group_vec.first() {
                    // let mods_num=binding_group.modifiers.len();

                    for (mapping2,binding_group2) in primary_mapping_binding_group_vec.iter() {
                        if binding_group.modifiers.len() == binding_group2.modifiers.len() {
                            founds.push((mapping2.clone(),binding_group2.clone(),))
                        }
                    }
                }
            }

            //
            for (mapping,binding_group) in founds {
                let Some(mapping_val) = mapping_vals.get_mut(&mapping) else { continue; };
                let binding_info=mapping_val.binding_infos.get(&binding_group).unwrap();

                //get/init binding_vals
                // let binding_vals=owner_mapping_binding_vals.entry((owner,mapping.clone())).or_insert_with(||mapping_val.binding_vals.clone());

                //get last binding val
                // let last_val=mapping_val.binding_vals.iter().map(|x|*x.1).sum::<f32>();
                let last_val=mapping_val.binding_vals.iter().map(|(_,&v)|v).sum::<f32>();
                let last_dir=if last_val>0.0{1}else if last_val<0.0{-1}else{0};

                //
                if binding_input.immediate { //ie mouse move/scroll
                    // if !modifiers_pressed { //what's this for?
                    //     panic!("input map, immediate, !modifiers_pressed");
                    // }

                    //
                    let cur_val=binding_input.value;
                    let cur_dir=if cur_val>0.0{1}else if cur_val<0.0{-1}else{0};

                    //
                    mapping_event_writer.send(InputMapEvent::TempValueChanged { mapping: mapping.clone(), val: cur_val, owner });

                    //reset repeating
                    if mapping_repeats.contains_key(&mapping) {
                        not_repeatings.insert((owner,mapping.clone()));
                    }

                    //send press/release events (cur_dir will never be 0)
                    if last_dir==cur_dir || last_dir!=0 { //(last_dir!=cur_dir && last_dir!=0)
                        mapping_event_writer.send(InputMapEvent::JustReleased { mapping: mapping.clone(), dir: last_dir, owner }); //0
                    }

                    if last_dir==0 || last_dir!=cur_dir { //(last_dir!=cur_dir && last_dir!=0)
                        mapping_event_writer.send(InputMapEvent::JustPressed{ mapping:mapping.clone(), dir: cur_dir, owner }); //1
                        mapping_event_writer.send(InputMapEvent::JustReleased { mapping: mapping.clone(), dir: cur_dir, owner }); //2
                    }

                    if last_dir==cur_dir || last_dir!=0 {
                        mapping_event_writer.send(InputMapEvent::JustPressed { mapping: mapping.clone(), dir: last_dir, owner }); //3
                    }
                } else {
                    //binding input val
                    let input_val = if binding_input.value.abs()<binding_info.primary_dead{0.0}else{binding_input.value}*binding_info.scale;
                    // let input_val = if modifiers_pressed {input_val} else {0.0};
                    mapping_val.binding_vals.insert((binding_input.device,binding_group.clone()),input_val);

                    //get cur val
                    // let cur_val=mapping_val.binding_vals.iter().map(|x|*x.1).sum::<f32>();
                    let cur_val=mapping_val.binding_vals.iter().map(|(_,&v)|v).sum::<f32>();
                    let cur_dir=if cur_val>0.0{1}else if cur_val<0.0{-1}else{0};

                    //change event
                    if last_val!=cur_val {
                        mapping_event_writer.send(InputMapEvent::ValueChanged { mapping: mapping.clone(), val: cur_val, owner });
                    }

                    //
                    if last_dir!=cur_dir {
                        //send press/release event
                        if cur_dir==0 || last_dir!=0 {
                            mapping_event_writer.send(InputMapEvent::JustReleased { mapping: mapping.clone(), dir: last_dir, owner });
                        }

                        if last_dir==0 || cur_dir!=0 {
                            mapping_event_writer.send(InputMapEvent::JustPressed { mapping: mapping.clone(), dir: cur_dir, owner });
                        }

                        //reset repeating
                        if mapping_repeats.contains_key(&mapping) {
                            not_repeatings.insert((owner,mapping.clone()));
                        }
                    }
                }
            }
        }
    } //for binding input

    //set disabled/reset repeats
    for (owner,mapping) in not_repeatings {
        let mapping_val=owner_mappings.get_mut(&owner).unwrap().get_mut(&mapping).unwrap();
        mapping_val.repeating=false;
    }

    //do repeatings
    for (mapping,&(repeat_initial_delay, repeat_time)) in mapping_repeats.iter() {
        for (&owner,mapping_vals) in owner_mappings.iter_mut() {
            let Some(mapping_val)=mapping_vals.get_mut(&mapping) else {continue;};
            // let cur_val:f32=mapping_val.binding_vals.iter().map(|x|*x.1).sum();
            let cur_val=mapping_val.binding_vals.iter().map(|(_,&v)|v).sum::<f32>();
            let cur_dir=if cur_val>0.0{1}else if cur_val<0.0{-1}else{0};

            if repeat_time<=0.0 || cur_val==0.0 { //cur_val, floating point errs? should clamp? eg clamp(val,-0.0001,0.0001)
                continue;
            }

            if mapping_val.repeating {
                let duration=repeat_time/cur_val.abs(); //why divide by cur_val? so repeats slower if value less than 1 (or faster if greater than 1, though generally 1 is max)
                mapping_val.repeat_time_accum+=time.delta_secs();

                if mapping_val.repeat_time_accum>=duration {
                    mapping_event_writer.send(InputMapEvent::Repeat {
                        mapping: mapping.clone(),
                        dir: cur_dir,
                        delay: duration,
                        owner,
                    });

                    mapping_val.repeat_time_accum=0.0;
                    // let dif=mapping_val.repeat_time_accum-duration;
                    // mapping_val.repeat_time_accum=dif-(dif/duration)*duration;
                }
            } else {
                if mapping_val.repeat_time_accum < repeat_initial_delay {
                    mapping_val.repeat_time_accum+=time.delta_secs();
                } else {
                    mapping_val.repeating=true;
                    mapping_val.repeat_time_accum=0.0;
                }
            }
        }
    }

    //do bind mode
    for binding_input in binding_inputs.iter() {
        // let Some(owner)=device_owner.get(&binding_input.device).cloned() else { continue; };
        // let owner=device_owner.get(&binding_input.device).cloned();
        let is_bind_mode= bind_mode_devices.contains(&binding_input.device);
        // let is_bind_mode= is_bind_mode && !bind_mode_excludes.contains(&binding_input.binding);
        // let is_bind_mode= is_bind_mode && is_binding_bind_mode(owner,&bind_mode_owner_excludes,&bind_mode_owner_includes,binding_input.binding);
        let is_bind_mode= is_bind_mode && is_binding_bind_mode(&bind_mode_excludes,&bind_mode_includes,binding_input.binding);

        if !is_bind_mode {
            continue;
        }

        let device_binding=(binding_input.device,binding_input.binding);
        let has_binding = bind_mode_bindings.contains(&device_binding);

        if !has_binding && binding_input.value.abs()>*bind_mode_start_dead {
            let chain_bindings=bind_mode_chain.entry(binding_input.device).or_default();
            chain_bindings.push(binding_input.binding);

            mapping_event_writer.send(InputMapEvent::BindPressed{
                // owner,
                device:binding_input.device,
                bindings:chain_bindings.clone(),
            });
            bind_mode_bindings.insert(device_binding);
        } else if has_binding && binding_input.value.abs()<*bind_mode_end_dead {
            let chain_bindings=bind_mode_chain.remove(&binding_input.device).unwrap();

            for &binding in chain_bindings.iter() {
                bind_mode_bindings.remove(&(binding_input.device,binding));
            }

            mapping_event_writer.send(InputMapEvent::BindReleased{
                // owner,
                device:binding_input.device,
                bindings:chain_bindings,
            });
        }
    }


    //calc device last bindmode
    device_bind_mode_lasts.clear();

    for &device in bind_mode_devices.iter() {
        device_bind_mode_lasts.insert(device);
    }
}

/*

if a press and release in the same step,

what if dif bindings one axis pos, and another neg, for the same mapping

*/