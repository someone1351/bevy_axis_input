
use std::{collections::{HashMap, HashSet}, fmt::Debug, hash::Hash};

use bevy::{ecs::prelude::*, prelude::GamepadAxis};
use bevy::input::gamepad::{GamepadAxisChangedEvent, GamepadButtonChangedEvent, GamepadConnection, GamepadConnectionEvent, GamepadEvent,};
use bevy::input::keyboard::KeyCode;

use super::resources::*;
use super::events::*;
use super::values::*;

fn use_dead_zone(value:f32,dead_zone:Option<&InputDeviceDeadZone>) -> f32 {
    let Some(dead_zone)=dead_zone else {
        return value;
    };

    let pos_min=dead_zone.pos_min.max(dead_zone.neg_min);
    let neg_min=dead_zone.neg_min.min(dead_zone.pos_min);
    let pos_max=dead_zone.pos_max.min(pos_min);
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

pub fn binding_inputs_system<M: Send + Sync + 'static + Eq + Debug>
(
    mut gamepad_events: EventReader<GamepadEvent>,
    mut key_events: EventReader<bevy::input::keyboard::KeyboardInput>,
    mut mouse_move_events: EventReader<bevy::input::mouse::MouseMotion>,
    mut mouse_scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut mouse_button_events : EventReader<bevy::input::mouse::MouseButtonInput>,

    mut input_map : ResMut<InputMap<M>>,

    mut gamepad_axis_lasts : Local<HashMap<(Device,GamepadAxis),f32>>,
    mut key_lasts : Local<HashSet<KeyCode>>,

    mut binding_input_event_writer: EventWriter<BindingInputEvent>,
    mut mapping_event_writer: EventWriter<InputMapEvent<M>>,
) {
    //
    for event in gamepad_events.read() {
        let immediate=false;

        match event {
            GamepadEvent::Connection(GamepadConnectionEvent{gamepad,connection:GamepadConnection::Connected {
                name, vendor_id, product_id
            }})=> {
                //println!("{gamepad} {name:?} Connected", );

                let mut device_index=None;

                for (i,x) in input_map.gamepad_devices.iter().enumerate() {
                    if x.is_none() {
                        device_index=Some(i);
                    }
                }

                if device_index.is_none() {
                    device_index=Some(input_map.gamepad_devices.len());
                    input_map.gamepad_devices.push(None);
                }

                *input_map.gamepad_devices.get_mut(device_index.unwrap()).unwrap()=Some((*gamepad,name.clone(),*vendor_id,*product_id));
                input_map.gamepad_device_entity_map.insert(*gamepad, device_index.unwrap());

                //


                mapping_event_writer.send(InputMapEvent::GamepadConnect{entity:*gamepad,index:device_index.unwrap(),name:name.clone(),vendor_id:*vendor_id, product_id:*product_id});

            }
            GamepadEvent::Connection(GamepadConnectionEvent{gamepad,connection:GamepadConnection::Disconnected})=> {
                //println!("{:?} Disconnected", gamepad);

                let &index=input_map.gamepad_device_entity_map.get(gamepad).unwrap();
                let (_,name,vendor_id,product_id) = input_map.gamepad_devices.get(index).cloned().unwrap().unwrap();

                mapping_event_writer.send(InputMapEvent::GamepadDisconnect{entity:*gamepad,index,name,vendor_id, product_id});

                // let i =input_map.gamepad_device_entity_map.remove(gamepad).unwrap();
                // *input_map.gamepad_devices.get_mut(i).unwrap()=None;
                //removal is done in system below
            }
            GamepadEvent::Button(GamepadButtonChangedEvent {value, entity, button:button_type, .. })=> {
                let device=Device::Gamepad(input_map.gamepad_device_entity_map.get(entity).cloned().unwrap());
                let binding=Binding::GamepadButton(*button_type);

                let dead_zone=input_map.device_dead_zones.get(&(device,binding));
                let value=use_dead_zone(*value,dead_zone);

                binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
            }
            GamepadEvent::Axis(GamepadAxisChangedEvent {value, entity, axis:axis_type })=> {
                let axis_type=*axis_type;
                let device=Device::Gamepad(input_map.gamepad_device_entity_map.get(entity).cloned().unwrap());
                let binding=Binding::GamepadAxis(axis_type);

                let dead_zone=input_map.device_dead_zones.get(&(device,binding));
                let value=use_dead_zone(*value,dead_zone);

                let last_value=gamepad_axis_lasts.get(&(device,axis_type)).cloned().unwrap_or_default();

                {
                    binding_input_event_writer.send(BindingInputEvent { device, immediate, binding, value, });
                }

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
    mut input_map : ResMut<InputMap<M>>,
    // mut bind_mode_event_writer: EventWriter<InputMapBindModeEvent>,
    mut mapping_event_writer: EventWriter<InputMapEvent<M>>,
    time: Res<bevy::time::Time>,

    mut modifier_binding_vals : Local<HashMap<(Device,Binding),f32>>,
) {

    let InputMap {
        player_bindings, player_bindings_updated,
        mapping_repeats,
        // device_dead_zones,
        player_mappings, player_primary_mappings, player_modifier_mappings,
        device_player,
        bind_mode_excludes, bind_mode_devices, bind_mode_bindings,
        // bind_mode_start_dead, bind_mode_end_dead,
        gamepad_devices, gamepad_device_entity_map,
        bind_mode_chain,
        ..
    }=input_map.as_mut();

    //
    if *player_bindings_updated {
        *player_bindings_updated=false;

        for (&player,mappings) in player_bindings.iter() {
            let mut temp_player_mappings: HashMap<M, HashMap<BindingGroup,MappingBindingInfo>>=HashMap::new();

            //collect input in temp mappings
            // for (mapping,bindings,scale,primary_dead,modifier_dead) in mappings.into_iter()
            for ((mapping, bindings),&(scale, primary_dead, modifier_dead)) in mappings.iter() {

                // let (mapping,bindings,scale,primary_dead,modifier_dead)=x; //x.borrow().clone();

                if bindings.is_empty() {
                    continue;
                }

                let temp_bindings=temp_player_mappings.entry(mapping.clone()).or_default();
                // let mut modifiers=bindings.to_vec();
                // let primary=modifiers.pop().unwrap();

                // let modifiers=HashSet::from(modifiers);
                let binding_group=BindingGroup{ modifiers: bindings[0..bindings.len()-1].to_vec(), primary: bindings.last().unwrap().clone() };

                temp_bindings.insert(binding_group,MappingBindingInfo{scale,primary_dead,modifier_dead}); //,binding_val:0.0
            }

            //setup primary binding mappings
            {
                let primary_mappings=player_primary_mappings.entry(player).or_default();
                primary_mappings.clear();

                for (mapping,temp_bindings) in temp_player_mappings.iter() {
                    for (bind_group,_) in temp_bindings.iter() {
                        primary_mappings.entry(bind_group.primary).or_default().insert((mapping.clone(),bind_group.clone()));
                    }
                }
                // println!("binding_mappings {binding_mappings:?}");
            }

            //setup modifier binding mappings
            {
                let modifier_mappings=player_modifier_mappings.entry(player).or_default();
                modifier_mappings.clear();

                for (mapping,temp_bindings) in temp_player_mappings.iter() {
                    for (bind_group,_) in temp_bindings.iter() {
                        for &modifier in bind_group.modifiers.iter() {
                            modifier_mappings.entry(modifier).or_default().insert((mapping.clone(),bind_group.clone()));
                        }
                    }
                }

                // println!("modifier_mappings {modifier_mappings:?}");
            }

            //setup/insert mappings
            {
                let mappings=player_mappings.entry(player).or_insert_with(Default::default);

                //remove mappings from player_mappings not in temp
                mappings.retain(|k,_|temp_player_mappings.contains_key(k));

                //add new mappings/binding_infos or replace bindings in player_mapping from temp
                for (mapping,temp_bindings) in temp_player_mappings {
                    let mapping_val=mappings.entry(mapping).or_default();

                    //remove any cur binding valss that are no longer used
                    for k in mapping_val.binding_vals.keys().cloned().collect::<Vec<_>>() {
                        if !temp_bindings.contains_key(&k.1) {
                            mapping_val.binding_vals.remove(&k).unwrap();
                        }
                    }

                    //
                    mapping_val.binding_infos=temp_bindings;
                }
            }
        }

    }




    //clear player_bind_mode_bindings when bind mode turned off
    {
        // let player_bind_mode_devices=input_map.player_bind_mode_devices.clone();

        // for (player,bind_mode_bindings) in input_map.player_bind_mode_bindings.iter_mut() {
        //     bind_mode_bindings.retain(|(device,binding)|{
        //         mapping_event_writer.send(InputMapEvent::BindReleased{player:player,device:device.clone(),binding:binding.clone()});
        //         !player_bind_mode_devices.get(player).map(|x|x.contains(device)).unwrap_or_default()
        //     });
        // }
    }

    //get players by device
    // let device_players= {
    //     let mut device_players= HashMap::<Device,Vec<PlayerId>>::new();

    //     for (&player,devices) in input_map.player_devices.iter() {
    //         for &device in devices.iter() {
    //             device_players.entry(device).or_default().push(player);
    //         }
    //     }

    //     device_players
    // };

    //
    let mut player_mapping_binding_vals : HashMap<(i32,M),HashMap<(Device,BindingGroup),f32>> = Default::default();
    let mut not_repeatings : HashSet<(i32, M)> = Default::default();

    //release inputs of disconnected gamepad (todo)
    for event in gamepad_events.read() {
        let GamepadEvent::Connection(GamepadConnectionEvent{gamepad,connection:GamepadConnection::Disconnected})=event else {
            continue;
        };

        let gamepad_device=Device::Gamepad(gamepad_device_entity_map.get(gamepad).cloned().unwrap());

        //
        {
            let i =gamepad_device_entity_map.remove(gamepad).unwrap();
            *gamepad_devices.get_mut(i).unwrap()=None;
        }

        //
        // let Some(players)=device_players.get(&gamepad_device) else {
        //     continue;
        // };

        let Some(player)=device_player.get(&gamepad_device).cloned() else {
            continue;
        };

        // for &player in players.iter()
        {
            let Some(mappings)=player_mappings.get(&player) else {
                continue;
            };

            for (mapping,mapping_val) in mappings.iter() {

                //get/init binding_vals
                let binding_vals=player_mapping_binding_vals
                    .entry((player,mapping.clone()))
                    .or_insert_with(||mapping_val.binding_vals.clone());

                //
                let last_val=mapping_val.binding_vals.iter().map(|x|*x.1).sum::<f32>();
                let last_dir=if last_val>0.0{1}else if last_val<0.0{-1}else{0};

                //
                binding_vals.retain(|(device,_bind_group),_binding_val|{
                    if gamepad_device==*device {
                        false
                    } else {
                        true
                    }
                });

                //
                let cur_val=binding_vals.iter().map(|x|*x.1).sum::<f32>();
                let cur_dir=if cur_val>0.0{1}else if cur_val<0.0{-1}else{0};

                //
                if last_dir!=cur_dir {
                    //send press/release event
                    if cur_dir==0 || last_dir!=0 {
                        mapping_event_writer.send(InputMapEvent::JustReleased{ mapping: mapping.clone(), dir: last_dir, player: player });
                    }

                    if last_dir==0 || cur_dir!=0 {
                        mapping_event_writer.send(InputMapEvent::JustPressed{ mapping: mapping.clone(), dir: cur_dir, player: player });
                    }

                    //reset repeating
                    if mapping_repeats.contains_key(&mapping) {
                        not_repeatings.insert((player,mapping.clone()));
                    }
                }
            }
        }
    }

    //
    let binding_inputs=binding_input_events.read().map(|&x|x).collect::<Vec<_>>();

    //get modifier input vals
    for binding_input in binding_inputs.iter() {
        if binding_input.immediate {
            continue;
        }

        let k=(binding_input.device,binding_input.binding);

        if binding_input.value == 0.0 {
            modifier_binding_vals.remove(&k);
        } else {
            modifier_binding_vals.insert(k,binding_input.value);
        }
    }

    //
    let mut player_bind_mode_devices:HashMap<i32,HashSet<Device>> = HashMap::new();

    for &device in bind_mode_devices.iter() {
        // let Some(players)=device_players.get(&device) else {continue;};

        let Some(player)=device_player.get(&device).cloned() else {
            continue;
        };

        // for &player in players {
        player_bind_mode_devices.entry(player).or_default().insert(device);
        // }
    }

    //clear presseds on bind mode
    for (&player, devices) in player_bind_mode_devices.iter() {

        //
        let Some(mapping_vals) = player_mappings.get(&player) else {continue;};

        for (mapping,mapping_val) in mapping_vals.iter() {

            //get/init binding_vals
            let binding_vals=player_mapping_binding_vals
                .entry((player,mapping.clone()))
                .or_insert_with(||mapping_val.binding_vals.clone());

            //
            let last_val=binding_vals.iter().map(|x|*x.1).sum::<f32>();
            let last_dir=if last_val>0.0{1}else if last_val<0.0{-1}else{0};

            //remove bind_groups that have device in bindmode, and aren't excluded from it
            binding_vals.retain(|(device,bind_group),_|{
                if !devices.contains(device) || (
                    bind_mode_excludes.contains(&bind_group.primary) &&
                    bind_group.modifiers.len()==bind_group.modifiers.iter().filter(|&x|bind_mode_excludes.contains(x)).count()
                ) {
                    true
                } else {
                    false
                }
            });

            //
            let cur_val=binding_vals.iter().map(|x|*x.1).sum::<f32>();
            let cur_dir=if cur_val>0.0{1}else if cur_val<0.0{-1}else{0};

            //
            if last_dir!=cur_dir {
                //send press/release event
                if cur_dir==0 || last_dir!=0 {
                    mapping_event_writer.send(InputMapEvent::JustReleased{ mapping: mapping.clone(), dir: last_dir, player: player });
                }

                if last_dir==0 || cur_dir!=0 {
                    mapping_event_writer.send(InputMapEvent::JustPressed { mapping: mapping.clone(), dir: cur_dir, player: player });
                }

                //reset repeating
                if mapping_repeats.contains_key(&mapping) {
                    not_repeatings.insert((player,mapping.clone()));
                }
            }

        }
        // //
        // for device in devices.iter() {

        // }
    }

    //
    //on binding release, check all pressed binggroups, that use that modifier and remove/depress them
    for binding_input in binding_inputs.iter() {
        //
        if binding_input.value!=0.0 || binding_input.immediate {
            continue;
        }

        //
        // let Some(players)=device_players.get(&binding_input.device) else {
        //     continue;
        // };

        let Some(player)=device_player.get(&binding_input.device).cloned() else {
            continue;
        };

        //
        // for &player in players
        {
            //
            let Some(modifier_mappings) = player_modifier_mappings.get(&player)
                .and_then(|modifier_mappings|modifier_mappings.get(&binding_input.binding))
            else {
                continue;
            };

            //
            let Some(mapping_vals) = player_mappings.get(&player) else {
                continue;
            };

            //
            //find (mapping,device,bind_groups) that released input binding is a modifier of
            for (mapping,bind_group) in modifier_mappings.iter() {
                //get mapping val
                let mapping_val = mapping_vals.get(mapping).unwrap();

                //get/init binding_vals
                let binding_vals=player_mapping_binding_vals
                    .entry((player,mapping.clone()))
                    .or_insert_with(||mapping_val.binding_vals.clone());
                //
                let k=(binding_input.device,bind_group.clone());

                let binding_val=binding_vals.get(&k).cloned().unwrap_or_default();

                // println!("hmm {k:?} = {binding_val}");

                if binding_val!=0.0 {
                    binding_vals.remove(&k).unwrap();

                    //get last binding val
                    let cur_val=binding_vals.iter().map(|x|*x.1).sum::<f32>();
                    let cur_dir=if cur_val>0.0{1}else if cur_val<0.0{-1}else{0};

                    //
                    if cur_val!=binding_val {
                        mapping_event_writer.send(InputMapEvent::ValueChanged { mapping: mapping.clone(), val: cur_val, player: player });
                    }

                    if cur_val==0.0 {
                        mapping_event_writer.send(InputMapEvent::JustReleased { mapping: mapping.clone(), dir: cur_dir, player: player });
                    }

                //
                }
            }


        }
    }

    //primaries
    for binding_input in binding_inputs.iter() {
        //
        // let Some(players)=device_players.get(&binding_input.device) else {
        //     continue;
        // };

        let Some(player)=device_player.get(&binding_input.device).cloned() else {
            continue;
        };

        //
        // for &player in players
        {
            // let bind_mode_devices=input_map.player_bind_mode_devices.get(&player);

            let is_bind_mode=bind_mode_devices.contains(&binding_input.device); //.unwrap_or_default();

            if is_bind_mode && !bind_mode_excludes.contains(&binding_input.binding) {
                continue;
            }

            //
            let Some(primary_mappings) =
                player_primary_mappings.get(&player)
                .and_then(|binding_mappings|binding_mappings.get(&binding_input.binding))
            else {
                continue;
            };

            //
            let Some(mapping_vals) = player_mappings.get(&player) else {
                continue;
            };

            //
            //
            let found: Option<(M, BindingGroup,bool)>={

                //check if any prev pressed
                let prev_binds=primary_mappings.iter().filter(|(mapping,bind_group)|{
                    //get mapping val
                    let mapping_val = mapping_vals.get(mapping).unwrap();

                    //
                    let binding_val=mapping_val.binding_vals.get(&(binding_input.device,bind_group.clone())).cloned().unwrap_or_default();

                    //
                    binding_val!=0.0
                }).map(|x|x.clone()).collect::<Vec<_>>();

                if prev_binds.len()>1 {
                    panic!("input map, prev binds, more than 1, should only be 1");
                }

                if let Some((mapping,bind_group))=prev_binds.first() {
                    //get mapping val
                    let mapping_val = mapping_vals.get(mapping).unwrap();

                    //get binding info
                    let binding_info=mapping_val.binding_infos.get(bind_group).unwrap();

                    let modifier_pressed_count=bind_group.modifiers.iter().filter(|&&modifier_bind|{
                        let modifier_val=modifier_binding_vals.get(&(binding_input.device,modifier_bind)).cloned().unwrap_or_default();
                        let modifier_val = if modifier_val.abs()<binding_info.modifier_dead{0.0}else{modifier_val};
                        modifier_val==0.0
                    }).count();

                    let modifiers_pressed=modifier_pressed_count==bind_group.modifiers.len();


                    Some((mapping.clone(),bind_group.clone(),modifiers_pressed))
                } else {

                    //get valid binding mappings
                    let mut binding_mappings=primary_mappings.iter().map(|x|x.clone()).collect::<Vec<_>>();
                    binding_mappings.sort_by(|a,b|b.1.modifiers.len().cmp(&a.1.modifiers.len()));

                    binding_mappings.retain(|(mapping,bind_group)|{
                        //get mapping val
                        let mapping_val = mapping_vals.get(mapping).unwrap();

                        //get binding info
                        let binding_info=mapping_val.binding_infos.get(bind_group).unwrap();

                        //check modifiers pressed
                        for &modifier_bind in bind_group.modifiers.iter() {
                            let modifier_val=modifier_binding_vals.get(&(binding_input.device,modifier_bind)).cloned().unwrap_or_default();
                            let modifier_val = if modifier_val.abs()<binding_info.modifier_dead{0.0}else{modifier_val};

                            if modifier_val== 0.0 {
                                return false;
                            }

                            if is_bind_mode && !bind_mode_excludes.contains(&modifier_bind)
                            {
                                return false;
                            }

                        }

                        //
                        true
                    });

                    binding_mappings.first().map(|(mapping,bind_group)|(mapping.clone(),bind_group.clone(),true))
                }
            };


            //
            if let Some((mapping,bind_group,modifiers_pressed))=found {

                //get mapping val
                let Some(mapping_val) = mapping_vals.get(&mapping) else {
                    continue;
                };

                //get binding info
                let binding_info=mapping_val.binding_infos.get(&bind_group).unwrap();

                //

                //get/init binding_vals
                let binding_vals=player_mapping_binding_vals
                    .entry((player,mapping.clone()))
                    .or_insert_with(||mapping_val.binding_vals.clone());

                //get last binding val
                let last_val=binding_vals.iter().map(|x|*x.1).sum::<f32>();
                let last_dir=if last_val>0.0{1}else if last_val<0.0{-1}else{0};

                // println!("==m {mapping:?}");

                //
                if binding_input.immediate {
                    if !modifiers_pressed {
                        panic!("input map, immediate, !modifiers_pressed");
                    }

                    //
                    let cur_val=binding_input.value;
                    let cur_dir=if cur_val>0.0{1}else if cur_val<0.0{-1}else{0};

                    //
                    mapping_event_writer.send(InputMapEvent::TempValueChanged { mapping: mapping.clone(), val: cur_val, player: player });

                    //reset repeating
                    if mapping_repeats.contains_key(&mapping) {
                        not_repeatings.insert((player,mapping.clone()));
                    }

                    //send press/release events (cur_dir will never be 0)
                    if last_dir==cur_dir || last_dir!=0 { //(last_dir!=cur_dir && last_dir!=0)
                        mapping_event_writer.send(InputMapEvent::JustReleased { mapping: mapping.clone(), dir: last_dir, player: player }); //0
                    }

                    if last_dir==0 || last_dir!=cur_dir { //(last_dir!=cur_dir && last_dir!=0)
                        mapping_event_writer.send(InputMapEvent::JustPressed{ mapping:mapping.clone(), dir: cur_dir, player: player }); //1
                        mapping_event_writer.send(InputMapEvent::JustReleased { mapping: mapping.clone(), dir: cur_dir, player: player }); //2
                    }

                    if last_dir==cur_dir || last_dir!=0 {
                        mapping_event_writer.send(InputMapEvent::JustPressed { mapping: mapping.clone(), dir: last_dir, player: player }); //3
                    }
                } else {
                    //binding input val
                    let input_val = if binding_input.value.abs()<binding_info.primary_dead{0.0}else{binding_input.value}*binding_info.scale;
                    let input_val = if modifiers_pressed {input_val} else {0.0};
                    binding_vals.insert((binding_input.device,bind_group.clone()),input_val);

                    //get cur val
                    let cur_val=binding_vals.iter().map(|x|*x.1).sum::<f32>();
                    let cur_dir=if cur_val>0.0{1}else if cur_val<0.0{-1}else{0};

                    //change event
                    if last_val!=cur_val {
                        mapping_event_writer.send(InputMapEvent::ValueChanged { mapping: mapping.clone(), val: cur_val, player: player });
                    }

                    //
                    if last_dir!=cur_dir {
                        //send press/release event
                        if cur_dir==0 || last_dir!=0 {
                            mapping_event_writer.send(InputMapEvent::JustReleased { mapping: mapping.clone(), dir: last_dir, player: player });
                        }

                        if last_dir==0 || cur_dir!=0 {
                            mapping_event_writer.send(InputMapEvent::JustPressed { mapping: mapping.clone(), dir: cur_dir, player: player });
                        }

                        //reset repeating
                        if mapping_repeats.contains_key(&mapping) {
                            not_repeatings.insert((player,mapping.clone()));
                        }

                    }

                    // if last_dir!=cur_dir || cur_dir==0 { //(cur_dir==0 && last_dir==0)

                    // }

                    //
                }
            }
        } //for player
    } //for binding input


    //store updated binding vals
    for ((player,mapping),binding_vals) in player_mapping_binding_vals {
        let mapping_val=player_mappings.get_mut(&player).unwrap().get_mut(&mapping).unwrap();
        mapping_val.binding_vals=binding_vals;
    }

    //set disabled/reset repeats
    for (player,mapping) in not_repeatings {
        let mapping_val=player_mappings.get_mut(&player).unwrap().get_mut(&mapping).unwrap();
        mapping_val.repeating=false;
    }

    //
    let mapping_repeats=mapping_repeats.clone();

    //do repeatings
    for (mapping,repeat_time) in mapping_repeats //input_map.mapping_repeats.iter()
    {
        for (&player,mapping_vals) in player_mappings.iter_mut() {
            let Some(mapping_val)=mapping_vals.get_mut(&mapping) else {continue;};

            // let cur_val:f32=mapping_val.binding_vals.iter().map(|x|*x.1).sum();
            let cur_val:f32=mapping_val.binding_vals.iter().map(|x|*x.1).sum();
            let cur_dir=if cur_val>0.0{1}else if cur_val<0.0{-1}else{0};

            if repeat_time<=0.0 {
                continue;
            }

            if cur_val==0.0 { //floating point errs? should clamp? eg clamp(val,-0.0001,0.0001)
                continue;
            }

            //
            if mapping_val.repeating {
                let duration=repeat_time/cur_val.abs();
                mapping_val.repeat_time_accum+=time.delta_secs();

                if mapping_val.repeat_time_accum>=duration {
                    mapping_event_writer.send(InputMapEvent::Repeat { mapping: mapping.clone(), dir: cur_dir, delay: duration, player: player });
                    mapping_val.repeat_time_accum=0.0;
                    // let dif=mapping_val.repeat_time_accum-duration;
                    // mapping_val.repeat_time_accum=dif-(dif/duration)*duration;
                }
            } else {
                mapping_val.repeating=true;
                mapping_val.repeat_time_accum=0.0;
            }
        }
    }

    //

    // let mut player_bind_mode_bindings : HashMap<PlayerId,HashSet<(Device,Binding)>> = Default::default();

    // let mut sbind_mode_bindings : HashSet<(Device,Binding)> = input_map.bind_mode_bindings.clone();

    //do bind mode
    for binding_input in binding_inputs.iter() {
        //
        // let Some(players)=device_players.get(&binding_input.device) else {
        //     continue;
        // };

        //
        // for &player in players
        {
            // let bind_mode_devices=input_map.player_bind_mode_devices.get(&player);

            let is_bind_mode=bind_mode_devices.contains(&binding_input.device); //.unwrap_or_default();

            if !is_bind_mode || bind_mode_excludes.contains(&binding_input.binding) {
                continue;
            }

            //
            // let bind_mode_bindings=input_map.player_bind_mode_bindings.get(&player);

            //get/init bind_mode_bindings
            // let bind_mode_bindings=player_bind_mode_bindings.entry(player).or_insert_with(||{
            //     input_map.player_bind_mode_bindings.get(&player).map(|x|x.clone()).unwrap_or_default()
            // });

            //
            let k=(binding_input.device,binding_input.binding);
            let has_binding = bind_mode_bindings.contains(&k);

            let Some(player)=device_player.get(&binding_input.device).cloned() else {
                continue;
            };

            if binding_input.value!=0.0 && !has_binding {
                // println!("a {:?} {:?} : {} {} : {bind_mode_bindings:?}",binding_input.device,binding_input.binding,binding_input.value, has_binding);

                // for &player in device_players.get(&binding_input.device).unwrap().iter() {
                mapping_event_writer.send(InputMapEvent::BindPress{player:player,device:binding_input.device,binding:binding_input.binding});
                // }

                bind_mode_bindings.insert(k);

                bind_mode_chain.entry(binding_input.device).or_default().push(binding_input.binding);


            } else if binding_input.value==0.0 && has_binding {
                // println!("b {:?} {:?} : {} {} : {bind_mode_bindings:?}",binding_input.device,binding_input.binding,binding_input.value, has_binding);

                if let Some(chain_bindings)=bind_mode_chain.remove(&binding_input.device) {
                    for &binding in chain_bindings.iter() {
                        bind_mode_bindings.remove(&(binding_input.device,binding));
                    }
                    mapping_event_writer.send(InputMapEvent::BindRelease{player:player,device:binding_input.device,bindings:chain_bindings});
                }
                // for &player in device_players.get(&binding_input.device).unwrap().iter() {
                // mapping_event_writer.send(InputMapEvent::BindRelease{player:player,device:binding_input.device,binding:binding_input.binding});
                // }

                // bind_mode_bindings.remove(&k);
            } else {

                // println!("c {:?} {:?} : {} {} : {bind_mode_bindings:?}",binding_input.device,binding_input.binding,binding_input.value, has_binding);
            }
            //
            // let bind_mode_dead_start=input_map.bind_mode_dead_start;
            // let bind_mode_dead_end=input_map.bind_mode_dead_end;
            // // let binding_events_enabled=input_map.bind_mode_enabled;



            // if
            // if binding_input.value ! {

            // }

        }
    }

    //store updated bind_mode_bindings
    // input_map.bind_mode_bindings=bind_mode_bindings;

    // for (player,bind_mode_bindings) in player_bind_mode_bindings {
    //     let x=input_map.player_bind_mode_bindings.entry(player).or_default();
    //     x.clear();
    //     x.extend(bind_mode_bindings);
    // }
}

/*

if a press and release in the same step,

what if dif bindings one axis pos, and another neg, for the same mapping

*/