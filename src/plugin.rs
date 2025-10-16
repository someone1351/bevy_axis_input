
use bevy::ecs::prelude::*;
use bevy::input::InputSystems;


use super::systems::*;
use super::resources::*;

// use super::binding::*;
use super::messages::*;


#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputMapSystems;

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
            .add_message::<InputMapMessage<M>>()
            .add_message::<BindingInputMessage>()

            .add_systems(bevy::app::PreUpdate, (
                binding_inputs_system::<M>,
                mapping_event_system::<M>,
            ).chain().in_set(InputMapSystems).after(InputSystems)
            // .before(mapping_event_system::<M>))
            // .add_systems(Update,(mapping_event_system::<M>,)
            )
            ;
    }
}
