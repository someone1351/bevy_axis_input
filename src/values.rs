use std::collections::HashMap;

use bevy::prelude::{ Entity, GamepadAxis, GamepadButton, KeyCode, MouseButton};

use serde::Deserialize;

#[derive(Hash, Eq, PartialEq, Clone, Copy,Debug)]
pub enum Device {
    // Touch,
    // MouseKeyboard
    Other,
    // Gamepad(usize), //GamepadId
    Gamepad(Entity), //GamepadId
}

#[derive(Clone, Hash, PartialEq, Eq,Debug)]
pub(super) struct BindingGroup {
    pub modifiers : Vec<Binding>,
    pub primary : Binding,
}

#[derive(Default)]
pub(super)struct MappingBindingInfo {
    pub scale : f32,
    pub primary_dead : f32,
    pub modifier_dead : f32,
    // pub binding_val : f32,
}

#[derive(Default)]
pub(super)struct MappingVal {
    pub binding_infos : HashMap<BindingGroup,MappingBindingInfo>,
    pub binding_vals:HashMap<(Device,BindingGroup),f32>,

    pub repeat_time_accum : f32, //system time
    pub repeating:bool,
}

#[derive(Default,Clone,Debug)]
pub struct DeadZone {
    pub pos_min : f32,
    pub pos_max : f32,
    pub neg_min : f32,
    pub neg_max: f32,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug,Deserialize,Default)]
pub enum Binding {
    #[default]
    None,

    MouseMoveX,
    MouseMoveY,

    MouseMovePosX,
    MouseMovePosY,
    MouseMoveNegX,
    MouseMoveNegY,

    MouseScrollPixelX,
    MouseScrollPixelY,


    MouseScrollPixelPosX,
    MouseScrollPixelPosY,
    MouseScrollPixelNegX,
    MouseScrollPixelNegY,

    MouseScrollLineX,
    MouseScrollLineY,

    MouseScrollLinePosX,
    MouseScrollLinePosY,

    MouseScrollLineNegX,
    MouseScrollLineNegY,


    GamepadAxisPos(GamepadAxis),
    GamepadAxisNeg(GamepadAxis),

    GamepadAxis(GamepadAxis),
    MouseButton(MouseButton),
    Key(KeyCode),
    // ModifierKey(Vec<KeyCode>),
    GamepadButton(GamepadButton),
}

impl std::str::FromStr for Binding {
    type Err = ron::de::SpannedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ron::de::from_str::<Self>(s)
    }
}


impl ToString for Binding {
    fn to_string(&self) -> String {
        format!("{:?}",self)
    }
}

impl Binding {
    pub fn get_inner_string(&self) -> String {
        match self {
            Self::Key(x) => format!("{x:?}"),
            Self::GamepadAxis(x) => format!("{x:?}"),
            Self::GamepadButton(x) => format!("{x:?}"),
            Self::MouseButton(x) => format!("{x:?}"),
            x => x.to_string(),
        }
    }
    pub fn get_outer_string(&self) -> &str {
        match self {
            Self::Key(_) => "Key",
            Self::GamepadAxis(_) => "GamepadAxis",
            Self::GamepadButton(_) => "GamepadButton",
            Self::MouseButton(_) => "MouseButton",
            Self::MouseMoveX => "MouseMoveX",
            Self::MouseMoveY => "MouseMoveY",
            Self::MouseScrollPixelX => "MouseScrollX",
            Self::MouseScrollPixelY => "MouseScrollY",
            Self::MouseScrollLineX => "MouseScrollLineX",
            Self::MouseScrollLineY => "MouseScrollLineY",

            Self::MouseMovePosX => "MouseMovePosX",
            Self::MouseMovePosY => "MouseMovePosY",
            Self::MouseMoveNegX => "MouseMoveNegX",
            Self::MouseMoveNegY => "MouseMoveNegY",

            Self::MouseScrollPixelPosX => "MouseScrollPosX",
            Self::MouseScrollPixelPosY => "MouseScrollPosY",
            Self::MouseScrollPixelNegX => "MouseScrollNegX",
            Self::MouseScrollPixelNegY => "MouseScrollNegY",

            Self::MouseScrollLinePosX => "MouseScrollLinePosX",
            Self::MouseScrollLinePosY => "MouseScrollLinePosY",
            Self::MouseScrollLineNegX => "MouseScrollLineNegX",
            Self::MouseScrollLineNegY => "MouseScrollLineNegY",

            Self::GamepadAxisPos(_) => "GamepadAxisPos",
            Self::GamepadAxisNeg(_) => "GamepadAxisNeg",
            Self::None => "None",

        }
    }
}
