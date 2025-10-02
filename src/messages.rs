use bevy::prelude::Message;
use std::fmt::Debug;
use super::values::*;



#[derive(Debug,Clone,PartialEq,Message,Copy)]

pub struct BindingInputMessage {
    pub binding : Binding,
    pub device : Device,
    pub immediate:bool,
    pub value : f32,
}


#[derive(Debug,Clone,PartialEq,Message)]//Copy,

pub enum InputMapMessage<M:Debug> {
    // GamepadConnect{entity:Entity,index:usize,name:String,vendor_id:Option<u16>, product_id:Option<u16>},
    // GamepadDisconnect{entity:Entity,index:usize,name:String,vendor_id:Option<u16>, product_id:Option<u16>},
    Repeat{mapping:M, dir:i32,delay:f32, owner:i32},
    JustPressed{mapping:M, dir:i32, owner:i32},
    JustReleased{mapping:M, dir:i32, owner:i32},
    ValueChanged{mapping:M, val:f32, owner:i32},
    TempValueChanged{mapping:M, val:f32, owner:i32},

    // BindPressed{owner:Option<i32>, device : Device, bindings : Vec<Binding>, },
    BindPressed{device : Device, bindings : Vec<Binding>, },
    //BindReleased{player:i32, device : Device, binding : Binding, },
    // BindReleased{owner:Option<i32>, device : Device, bindings : Vec<Binding>, },
    BindReleased{device : Device, bindings : Vec<Binding>, },
}

// impl<M:Copy+Debug> InputMapEvent<M> {
//     pub fn forward<T:Debug>(&self,f : fn(M)->Option<T>)->Option<InputMapEvent<T>> {
//         match *self {
//             Self::Repeat(p,m,s,t)=>f(m).and_then(|x|Some(InputMapEvent::Repeat(p,x,s,t))),
//             Self::JustPressed(p,m,s)=>f(m).and_then(|x|Some(InputMapEvent::JustPressed(p,x,s))),
//             Self::JustReleased(p,m,s)=>f(m).and_then(|x|Some(InputMapEvent::JustReleased(p,x,s))),
//             Self::ValueChanged(p,m,v)=>f(m).and_then(|x|Some(InputMapEvent::ValueChanged(p,x,v))),
//             Self::TempValueChanged(p,m,v)=>f(m).and_then(|x|Some(InputMapEvent::TempValueChanged(p,x,v))),
//             Self::BindPressed{..}=>None, //{device,binding}f(m).and_then(|x|Some(InputMapEvent::BindPressed{device,binding})),
//             Self::BindReleased{..}=>None, //{device,binding}f(m).and_then(|x|Some(InputMapEvent::BindReleased{device,binding})),
//         }
//     }
// }

