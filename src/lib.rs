/*
+add global sensitivity setting, and also per bindings, and per mapping?
+make mapping bindings that have more than one binding, to require pressing the first buttons
in the array first eg ctrl+f, ctrl first
+also if a mapping bindings like ctrl+f is pressed, then another mapping binding with only f isn't counted as pressed

*/
//check if cur_time - last_repeat_time > repeat_value*binding_value, then send repeat msg, don't send multiple to catch up

/*
TODO
* need "options" mode, to stop the gamepad stick from sending both vert and horizontal inputs
* * could increase dead zone just for ui

* touch mode
* * pan (1-2 fingeer drag), pinch, onscreen stick, tap

* handle modifier + key
- ctrl,alt,shift,win
- binding a modifier+key,
- binding only a modifier key, would require the binding be sent on the key release
- let any key (except same) be used as modifier?
- have multiple or single modifier key? if multiple,
- - have to be done in order
- - only send key press when all modifier keys pressed
- - send key release when any of the keys released
- - allow mixed modifiers eg keys, mouse buttons, gamepad buttons
- - value should be of the last binding presssed, eg button1+button2+axis1 => axis1
- - when any key is release during binding, that marks the end of the bind
- - - use 0.5 dead zone when bindings

* InputMapBindingEvent
* * add hashset of modifiers,
* * * have any bindings (of same device?) currently pressed added to modifier hashset
* * check when any of pressed bindevents have been released after being pressed?

* sticky/toggleable modifiers?
*/
mod resources;
mod systems;
mod values;
mod messages;
mod plugin;
mod components;

pub use resources::*;
pub use values::*;
pub use messages::*;
pub use components::*;

pub use plugin::*;
// pub use resources::InputMap;