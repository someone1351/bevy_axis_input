
use bevy::ecs::prelude::*;


use super::systems::*;
use super::resources::*;

// use super::binding::*;
use super::events::*;


#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputMapSystem;

pub struct InputMapPlugin<M : 'static>(std::marker::PhantomData<&'static M>);

impl<M> Default for InputMapPlugin<M> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<M> bevy::app::Plugin for InputMapPlugin<M> 
where
    InputMap<M>: Default,
    M: std::hash::Hash + Eq + std::fmt::Debug + Clone + Send + Sync,
{
    fn build(&self, app: &mut bevy::app::App) {
        app
            .init_resource::<InputMap<M>>()
            .add_event::<InputMapEvent<M>>()
            .add_event::<BindingInputEvent>()
            
            .add_systems(bevy::app::PreUpdate, (
                binding_inputs_system::<M>,
                mapping_event_system::<M>,
            ).chain().in_set(InputMapSystem).after(bevy::input::InputSystem)
            // .before(mapping_event_system::<M>))
            // .add_systems(Update,(mapping_event_system::<M>,)
            )
            ;
    }
}
