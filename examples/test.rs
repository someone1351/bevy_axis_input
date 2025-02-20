
use bevy::prelude::*;
use bevy_axis_input::{self as axis_input, Binding, SetMappingBind, SetMappingRepeat};
use serde::Deserialize;

#[derive(Clone,Debug,Deserialize,Hash,PartialEq,Eq,Ord,PartialOrd)]
pub enum Mapping {
    X,Y,
    Quit,
    MenuSelect,
    MenuCancel,
    MenuUp,
}

#[derive(Resource,)]
struct MappingBinds {
    items : Vec<SetMappingBind<Mapping>>,
    x_pos : SetMappingBind<Mapping>,
    x_neg : SetMappingBind<Mapping>,
    y : SetMappingBind<Mapping>,
}

impl Default for MappingBinds {
    fn default() -> Self {
        Self {
            items: vec![
                SetMappingBind{ mapping: Mapping::Quit, bindings: vec![Binding::Key(KeyCode::F4)], scale: 1.0, primary_dead: 0.0, modifier_dead: 0.0 },
                SetMappingBind{ mapping: Mapping::MenuUp, bindings: vec![Binding::Key(KeyCode::ArrowUp)], scale: 1.0, primary_dead: 0.0, modifier_dead: 0.0 },
                SetMappingBind{ mapping: Mapping::MenuUp, bindings: vec![Binding::Key(KeyCode::ArrowDown)], scale: -1.0, primary_dead: 0.0, modifier_dead: 0.0 },
                SetMappingBind{ mapping: Mapping::MenuSelect, bindings: vec![Binding::Key(KeyCode::Enter)], scale: 1.0, primary_dead: 0.0, modifier_dead: 0.0 },
                SetMappingBind{ mapping: Mapping::MenuCancel, bindings: vec![Binding::Key(KeyCode::Escape)], scale: 1.0, primary_dead: 0.0, modifier_dead: 0.0 },
            ],
            x_pos: SetMappingBind{ mapping: Mapping::X, bindings: vec![Binding::Key(KeyCode::KeyW)], scale: 1.0, primary_dead: 0.0, modifier_dead: 0.0 },
            x_neg: SetMappingBind{ mapping: Mapping::X, bindings: vec![Binding::Key(KeyCode::KeyS)], scale: -1.0, primary_dead: 0.0, modifier_dead: 0.0 },
            y: SetMappingBind{ mapping: Mapping::Y, bindings: vec![Binding::Key(KeyCode::Space)], scale: 1.0, primary_dead: 0.0, modifier_dead: 0.0 },
        }
    }
}

impl MappingBinds {
    fn get_items(&self) -> Vec<SetMappingBind<Mapping>> {
        let mut mapping_binds_items=self.items.clone();
        mapping_binds_items.extend([
            self.x_pos.clone(),
            self.x_neg.clone(),
            self.y.clone(),
        ]);

        mapping_binds_items
    }
}

#[derive(Resource,Default)]
struct Menu {
    cur_index : i32,
    pressed : Option<i32>,
    x_val : f32,
    y_val : f32,
    in_bind_mode:bool,
}

fn main() {
    let mut app = App::new();

    app
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {watch_for_changes_override:Some(true), ..default() })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "some input map".into(),
                        resolution: (800.0, 600.0).into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                }),
                axis_input::InputMapPlugin::<Mapping>::default(),
        ))

        .init_resource::<MappingBinds>()
        .init_resource::<Menu>()

        .add_systems(Startup, ( setup_input, setup_camera, setup_menu, ))
        .add_systems(PreUpdate, ( update_input, ).after(axis_input::InputMapSystem))
        .add_systems(Update, ( show_menu, ))
        ;

    app.run();
}

fn setup_input(
    mut input_map: ResMut<axis_input::InputMap<Mapping>>,
    mapping_binds : Res<MappingBinds>,
) {
    input_map.set_mapping_repeats([SetMappingRepeat{ mapping: Mapping::MenuUp, delay: 0.3 }]);
    input_map.set_player_devices(0, [axis_input::Device::Other,axis_input::Device::Gamepad(0)]);

    input_map.set_bind_mode_excludes([
        Binding::Key(KeyCode::Escape),
        Binding::Key(KeyCode::F4),
        Binding::Key(KeyCode::ArrowUp),
        Binding::Key(KeyCode::ArrowDown),
        Binding::Key(KeyCode::Enter),

        //so it uses GamepadAxisPos, GamepadAxisNeg instead
        Binding::GamepadAxis(GamepadAxis::LeftStickX),
        Binding::GamepadAxis(GamepadAxis::LeftStickY),
        Binding::GamepadAxis(GamepadAxis::RightStickX),
        Binding::GamepadAxis(GamepadAxis::RightStickY),
    ]);

    input_map.set_player_mapping_binds(0, mapping_binds.get_items());
}

fn update_input(
    mut input_map_event: EventReader<axis_input::InputMapEvent<Mapping>>,
    mut exit: EventWriter<AppExit>,
    mut menu : ResMut<Menu>,
    mut mapping_binds : ResMut<MappingBinds>,
    mut input_map: ResMut<axis_input::InputMap<Mapping>>,
) {
    for ev in input_map_event.read() {
        match ev.clone() {
            axis_input::InputMapEvent::GamepadConnect { entity, index, name, vendor_id, product_id } => {
                println!("Gamepad connected: {entity} {index} {name:?} {vendor_id:?} {product_id:?}");
            }
            axis_input::InputMapEvent::GamepadDisconnect { entity, index, name, vendor_id, product_id } => {
                println!("Gamepad disconnected: {entity} {index} {name:?} {vendor_id:?} {product_id:?}");
            }
            axis_input::InputMapEvent::ValueChanged { mapping:Mapping::X, val, .. } => {
                menu.x_val=val;
            }
            axis_input::InputMapEvent::ValueChanged { mapping:Mapping::Y, val, .. } => {
                menu.y_val=val;
            }
            axis_input::InputMapEvent::JustPressed{mapping:Mapping::Quit, ..} => {
                exit.send(AppExit::Success);
            }
            axis_input::InputMapEvent::JustPressed{mapping:Mapping::MenuUp, dir, ..}
                |axis_input::InputMapEvent::Repeat { mapping:Mapping::MenuUp, dir, .. }
                if !menu.in_bind_mode
            => {
                menu.cur_index-=dir;
                let n= 4;
                if menu.cur_index<0 {menu.cur_index=n-1;}
                if menu.cur_index==n {menu.cur_index=0;}
                menu.pressed=None;
            }
            axis_input::InputMapEvent::JustPressed{mapping:Mapping::MenuSelect, ..} => {
                menu.pressed=Some(menu.cur_index);
            }
            axis_input::InputMapEvent::JustReleased{mapping:Mapping::MenuSelect, ..} => {
                if let Some(pressed)=menu.pressed {
                    match pressed {
                        0..=2 => { //X+ X- Y
                            input_map.set_player_bind_mode_devices(0, [axis_input::Device::Other,axis_input::Device::Gamepad(0)]);
                            menu.in_bind_mode=true;
                            println!("bind mode start");
                        }
                        3 => { //Exit
                            exit.send(AppExit::Success);
                        }
                        _ =>{}
                    }
                }
                menu.pressed=None;
            }

            axis_input::InputMapEvent::BindPressed { player:0, binding, .. } => {
                input_map.set_player_bind_mode_devices(0, []);
                menu.in_bind_mode=false;

                match menu.cur_index {
                    0 => { //X+
                        mapping_binds.x_pos.bindings.clear();
                        mapping_binds.x_pos.bindings.push(binding);
                    }
                    1 => { //X-
                        mapping_binds.x_neg.bindings.clear();
                        mapping_binds.x_neg.bindings.push(binding);
                    }
                    2 => { //Y
                        mapping_binds.y.bindings.clear();
                        mapping_binds.y.bindings.push(binding);
                    }
                    _=>{}
                }
                input_map.set_player_mapping_binds(0, mapping_binds.get_items());
            }
            axis_input::InputMapEvent::JustPressed{mapping:Mapping::MenuCancel, ..} => {
                if menu.in_bind_mode {
                    input_map.set_player_bind_mode_devices(0, []);
                    menu.in_bind_mode=false;
                } else {
                    match menu.cur_index {
                        0 => { //X+
                            mapping_binds.x_pos.bindings.clear();
                            input_map.set_player_mapping_binds(0, mapping_binds.get_items());
                        }
                        1 => { //X-
                            mapping_binds.x_neg.bindings.clear();
                            input_map.set_player_mapping_binds(0, mapping_binds.get_items());
                        }
                        2 => { //Y
                            mapping_binds.y.bindings.clear();
                            input_map.set_player_mapping_binds(0, mapping_binds.get_items());
                        }
                        _ =>{}
                    }
                }
            }

            _=>{}
        }
    }
}

fn setup_camera(mut commands: Commands) {
    // commands.spawn((Camera2d,));
    commands.spawn(Camera3d::default());
}

#[derive(Component)]
struct MenuItem(i32);

fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("FiraMono-Medium.ttf");
    // // let text_bundle=TextBundle::from_section("", Default::default());
    // // let style=Style{align_self:AlignSelf::Center,justify_self:JustifySelf::Center,..Default::default()};
    // // commands.spawn(text_bundle.with_style(style)).insert(MenuMarker);

    // commands.spawn((
    //     Text::new("hello\nbevy!"),
    //     TextFont {
    //         font: asset_server.load("FiraMono-Medium.ttf"),
    //         font_size: 25.0,
    //         ..default()
    //     },
    //     TextLayout::new_with_justify(JustifyText::Center),
    //     Node {
    //         position_type: PositionType::Absolute,
    //         bottom: Val::Px(5.0),
    //         right: Val::Px(5.0),
    //         ..default()
    //     },
    //     MenuMarker,
    // ));
    commands.spawn((
        Text::default(),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {align_self:AlignSelf::Center,justify_self:JustifySelf::Center,..Default::default()},
    )).with_child((
        TextSpan::new("\"Press Up/Down to navigate, Enter to select, Escape to cancel/clear binding.\""),
        TextColor::from(bevy::color::palettes::css::WHITE),
        TextFont {font:font.clone(),font_size: 15.0,..default()},
    )).with_child((
        TextSpan::new("\n\n"),
        TextFont {font:font.clone(),font_size: 25.0,..default()},
    )).with_child((
    )).with_child((
        MenuItem(-1),
        TextSpan::new("values"),
        TextColor::from(bevy::color::palettes::css::WHITE),
        TextFont {font:font.clone(),font_size: 25.0,..default()},
    )).with_child((
        TextSpan::new("\n"),
        TextFont {font:font.clone(),font_size: 25.0,..default()},
    )).with_child((
        MenuItem(0),
        TextSpan::new("b\n"),
        TextColor::from(bevy::color::palettes::css::WHITE),
        TextFont {font:font.clone(),font_size: 25.0,..default()},
    )).with_child((
        MenuItem(1),
        TextSpan::new("b\n"),
        TextColor::from(bevy::color::palettes::css::WHITE),
        TextFont {font:font.clone(),font_size: 25.0,..default()},
    )).with_child((
        MenuItem(2),
        TextSpan::new("b\n"),
        TextColor::from(bevy::color::palettes::css::WHITE),
        TextFont {font:font.clone(),font_size: 25.0,..default()},
    )).with_child((
        MenuItem(3),
        TextSpan::new("Exit"),
        TextColor::from(bevy::color::palettes::css::WHITE),
        TextFont {font:font.clone(),font_size: 25.0,..default()},
    ));
}

fn show_menu(
    mut marker_query: Query<(&MenuItem, &mut TextSpan, &mut TextColor)>,
    menu : Res<Menu>,
    mapping_binds : Res<MappingBinds>,
) {
    // let text_style = TextFont{ font, font_size:25.0 , ..Default::default()}; //, color: Color::WHITE

    for (item,mut text,mut col) in marker_query.iter_mut() {

        if item.0==menu.cur_index {
            col.0=Color::linear_rgb(1.0, 0.0, 0.0);
        } else {
            col.0=Color::linear_rgb(1.0,1.0,1.0);
        }

        if let Some(i)=menu.pressed {
            if item.0==i {
                col.0=Color::linear_rgb(0.8, 0.8, 0.0);
            } else {
                col.0=Color::linear_rgb(1.0,1.0,1.0);
            }
        }

        match item.0 {
            -1 => {
                text.0=format!("\"X={:.3}, Y={:.3}\"\n",menu.x_val,menu.y_val);
            }
            0 => {
                text.0=format!("Rebind X+ : {:?}\n",
                    if menu.in_bind_mode&&menu.cur_index==0 {
                        "...".to_string()
                    }else{
                        mapping_binds.x_pos.bindings.first().map(|x|format!("{x:?}")).unwrap_or_default()
                    }
                );
            }
            1 => {
                text.0=format!("Rebind X- : {:?}\n",
                    if menu.in_bind_mode&&menu.cur_index==1 {
                        "...".to_string()
                    }else{
                        mapping_binds.x_neg.bindings.first().map(|x|format!("{x:?}")).unwrap_or_default()
                    }
                );
            }
            2 => {
                text.0=format!("Rebind Y : {:?}\n",
                    if menu.in_bind_mode&&menu.cur_index==2 {
                        "...".to_string()
                    }else{
                        mapping_binds.y.bindings.first().map(|x|format!("{x:?}")).unwrap_or_default()
                    }
                );
            }
            _ => {}
        }
    }
    // if let Ok(mut text)=marker_query.get_single_mut() {
    //     text.sections.clear();

    //     text.justify =JustifyText::Center;
    //     text.sections.push(TextSection { value: "\"Press Up/Down to navigate, Enter to select, Escape to cancel/clear binding.\"\n".to_string(), style: TextStyle{font_size:20.0, ..text_style.clone()} }); //0
    //     text.sections.push(TextSection { value: "\n".to_string(), style: text_style.clone()}); //1
    //     text.sections.push(TextSection { value: format!("\"X={:.3}, Y={:.3}\"\n",menu.x_val,menu.y_val), style: TextStyle{font_size:20.0, ..text_style.clone()} }); //02
    //     text.sections.push(TextSection { value: "\n".to_string(), style: text_style.clone()}); //3

    //     text.sections.push(TextSection {
    //         value: format!("Rebind X+ : {:?}\n",
    //             if menu.in_bind_mode&&menu.cur_index==0 {"...".to_string()}else{mapping_binds.x_pos.bindings.first().map(|x|format!("{x:?}")).unwrap_or_default()}
    //         ),
    //         style: text_style.clone()}
    //     );

    //     text.sections.push(TextSection {
    //         value: format!("Rebind X- : {:?}\n",
    //             if menu.in_bind_mode&&menu.cur_index==1 {"...".to_string()}else{mapping_binds.x_neg.bindings.first().map(|x|format!("{x:?}")).unwrap_or_default()}
    //         ),
    //         style: text_style.clone()}
    //     );

    //     text.sections.push(TextSection {
    //         value: format!("Rebind Y : {:?}\n",
    //             if menu.in_bind_mode&&menu.cur_index==2 {"...".to_string()}else{mapping_binds.y.bindings.first().map(|x|format!("{x:?}")).unwrap_or_default()}
    //         ),
    //         style: text_style.clone()}
    //     );

    //     text.sections.push(TextSection { value: "Exit\n".to_string(), style: text_style.clone()});

    //     text.sections[(menu.cur_index as usize)+4].style.color=Color::linear_rgb(1.0, 0.0, 0.0);

    //     if let Some(i)=menu.pressed {
    //         text.sections[(i as usize)+4].style.color=Color::linear_rgb(0.8, 0.8, 0.0);
    //     }
    // }
}
