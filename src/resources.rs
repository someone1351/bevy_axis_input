
use std::collections::{ HashMap, HashSet};
use std::fmt::Debug;
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
    pub(super) mapping_repeats : HashMap<M,f32>, //[mapping]=repeat //
    pub(super) device_dead_zones : HashMap<Device,HashMap<Binding,InputBindingDeadZone>>, //

    pub(super) player_mappings : HashMap<PlayerId, HashMap<M,MappingVal>>, //[player][mapping]=mapping_val
    pub(super) player_primary_mappings : HashMap<PlayerId, HashMap<Binding,HashSet<(M,BindingGroup)>>>, //[player][primary_binding][(mapping,binding_group)]
    pub(super) player_modifier_mappings : HashMap<PlayerId, HashMap<Binding,HashSet<(M,BindingGroup)>>>, //[player][modifier_binding][(mapping,binding_group)]


    pub(super) player_devices : HashMap<PlayerId,HashSet<Device>>, //[player]=devices //
    pub(super) bind_mode_excludes : HashSet<Binding>, //[binding] //

    // pub(super) bind_mode_enabled:bool,

    //
    pub(super) bind_mode_devices:HashSet<Device>, //
    pub(super) bind_mode_bindings:HashSet<(Device,Binding)>,

    pub(super) bind_mode_dead_start:f32, //
    pub(super) bind_mode_dead_end:f32, //

    //

    pub(super) gamepad_devices:Vec<Option<(Entity,String,Option<u16>,Option<u16>)>>,
    pub(super) gamepad_device_entity_map:HashMap<Entity,usize>,
}

impl<M:Eq> Default for InputMap<M> {
    fn default() -> Self {
        Self {
            mapping_repeats:HashMap::new(),
            player_mappings : HashMap::new(),
            player_primary_mappings : HashMap::new(),
            player_modifier_mappings : HashMap::new(),
            player_devices : HashMap::new(),
            device_dead_zones : HashMap::new(),
            // bind_mode_enabled:false,
            // player_bind_mode_devices: HashMap::new(),
            bind_mode_devices: HashSet::new(),
            // player_bind_mode_bindings: HashMap::new(),
            bind_mode_bindings: HashSet::new(),
            bind_mode_dead_start:0.4,
            bind_mode_dead_end:0.2,
            bind_mode_excludes:HashSet::new(),
            gamepad_devices:Vec::new(),
            gamepad_device_entity_map:HashMap::new(),
        }
    }
}

//for binding, if multiple keys pressed, then last key pressed is the primary, and when any of them are released the binding is finished

//need to clear binding_val.player_mapping_bind_groups when set_player_devices, set_player_mapping_bindings ??

impl<M> InputMap<M>
where
    M: std::hash::Hash + Eq + Clone + Send + Sync+Debug, //+Ord
{

    pub fn set_player_devices<I> //<I,J>
        (&mut self,player:i32,devices:I) //[device]
    where
        // I:AsRef<[Device]>
        // I: IntoIterator<Item = J>,
        // J :std::borrow::Borrow<Device>,

        I: IntoIterator<Item = Device>,
        // I:FromIterator<Device>,
    {
        let player_devices=self.player_devices.entry(PlayerId(player)).or_default();
        player_devices.clear();

        // player_devices.extend(devices.as_ref());
        player_devices.extend(devices.into_iter());
    }

    pub fn set_bind_mode_dead(&mut self,start:f32,end:f32) {
        self.bind_mode_dead_start=start;
        self.bind_mode_dead_end=end;
    }

    pub fn set_bind_mode_excludes<I> //<I ,J>
        (&mut self,bindings:I) //[binding]
    where
        // I:AsRef<[Binding]>
        // I: IntoIterator<Item = J>,
        // J :std::borrow::Borrow<Binding>,
        // // J :AsRef<Binding>,

        I: IntoIterator<Item = Binding>,
    {
        self.bind_mode_excludes.clear();
        // self.bind_mode_excludes.extend(bindings.as_ref().into_iter().map(|x|x.clone()));
        self.bind_mode_excludes.extend(bindings.into_iter());

        // println!("hmm {:?}",self.bind_mode_excludes);
    }

    pub fn set_bind_mode_devices<I> //<I,J>
        (&mut self,
            //player:i32,
            devices:I)
    where
        // I:AsRef<[Device]>
        // I: IntoIterator<Item = J>,
        // J :std::borrow::Borrow<Device>,
        // J :std::ops::Deref<Target = Device>,
        // J :AsRef< Device>,
        // J :Clone,

        I: IntoIterator<Item=Device>,
    {
        // let bind_mode_devices=self.player_bind_mode_devices.entry(PlayerId(player)).or_default();
        self.bind_mode_devices.clear();
        // bind_mode_devices.extend(devices.as_ref());
        self.bind_mode_devices.extend(devices.into_iter());
        // bind_mode_devices.extend(devices.into_iter().map(|x|*x.borrow()));
        // bind_mode_devices.extend(devices.into_iter().map(|x|x));
        // bind_mode_devices.extend(devices.into_iter().map(|x|x.clone()));
    }

    pub fn set_player_mapping_binds<I> //<I,J>
        (&mut self,player:i32,mappings:I) //[(mapping,bindings,scale,primary_dead,modifier_dead)]
    where
        // // I:AsRef<[(M,Vec<Binding>,f32,f32,f32)]>
        // for<'a> &'a I: IntoIterator<Item = &'a MappingBind<M>>,
        // // I: IntoIterator<Item = J>,
        // // J :std::borrow::Borrow<(M,Vec<Binding>,f32,f32,f32)>,

        // I: IntoIterator<Item = (M,Vec<Binding>,f32,f32,f32)>,
        I: IntoIterator<Item = SetMappingBind<M>>,
    {
        let player=PlayerId(player);

        let mut temp_player_mappings: HashMap<M, HashMap<BindingGroup,MappingBindingInfo>>=HashMap::new();

        //collect input in temp mappings
        // for (mapping,bindings,scale,primary_dead,modifier_dead) in mappings.into_iter()
        for SetMappingBind{ mapping, bindings, scale, primary_dead, modifier_dead } in mappings.into_iter()
        {
            // let (mapping,bindings,scale,primary_dead,modifier_dead)=x; //x.borrow().clone();

            if bindings.is_empty() {
                continue;
            }

            let temp_bindings=temp_player_mappings.entry(mapping).or_default();
            let mut modifiers=bindings.to_vec();
            let primary=modifiers.pop().unwrap();

            // let modifiers=HashSet::from(modifiers);

            temp_bindings.insert(BindingGroup{modifiers,primary},MappingBindingInfo{scale,primary_dead,modifier_dead}); //,binding_val:0.0
        }

        //setup primary binding mappings
        {
            let primary_mappings=self.player_primary_mappings.entry(player).or_default();
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
            let modifier_mappings=self.player_modifier_mappings.entry(player).or_default();
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
            let mappings=self.player_mappings.entry(player).or_insert_with(Default::default);

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

    pub fn set_device_dead_zones<I> //<I,J>
        (&mut self,device:Device, binding_deads:I) //[(binding,neg,pos)]
    where
        // // I:AsRef<[(Binding,f32,f32)]>
        // // J :std::borrow::Borrow<(Binding,f32,f32)>,

        // I: IntoIterator<Item = (Binding,f32,f32)>,
        I: IntoIterator<Item = SetBindingDead>,
    {
    //     //axis:GamepadAxisType GamepadButtonType gamepad_id:usize
    //     //should only need to do for gamepad axises
    //     // but for some reason the GamepadAxisType::LeftZ, GamepadAxisType::RightZ do nothing
    //     // and the GamepadButtonType::LeftTrigger, GamepadButtonType::RightTrigger are used for the axis values

    //     let device_deads=self.device_dead_zones.entry(device).or_default();
    //     device_deads.clear();

    //     // device_deads.extend(binding_deads.as_ref().iter().map(|&(binding,neg,pos)|(binding,InputBindingDeadZone {neg,pos})));
    //     // device_deads.extend(binding_deads.into_iter().map(|(binding,neg,pos)|(binding,InputBindingDeadZone {neg,pos})));
    //     device_deads.extend(binding_deads.into_iter().map(|x|(x.binding,InputBindingDeadZone {neg:x.neg,pos:x.pos})));

    // //     device_deads.extend(binding_deads.into_iter()
    // //         .map(|x|x.borrow().clone())
    // //         .map(|(binding,neg,pos)|(binding,InputBindingDeadZone {neg,pos}))
    // //     );
    }

    pub fn set_mapping_repeats<I> //<I,J>
        (&mut self, mapping_repeats:I) //[(mapping,repeat)]
    where
        // // I:AsRef<[(M,f32)]>,
        // // I: IntoIterator<Item = J>,
        // // J :std::borrow::Borrow<(M,f32)>,

        // I: IntoIterator<Item = (M,f32)>,
        I: IntoIterator<Item = SetMappingRepeat<M>>,
    {
        self.mapping_repeats.clear();
        // self.mapping_repeats.extend(mapping_repeats.as_ref().iter().map(|x|x.clone()));
        // self.mapping_repeats.extend(mapping_repeats.into_iter().map(|x|x.borrow().clone()));

        // self.mapping_repeats.extend(mapping_repeats.into_iter());
        self.mapping_repeats.extend(mapping_repeats.into_iter().map(|x|(x.mapping,x.delay)));
    }

}
