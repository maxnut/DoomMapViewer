use crate::state::{GameState, Marker};
use crate::AppState;
use bevy::app::{App, Plugin};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use native_dialog::FileDialog;
use tinywad::wad::Wad;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub struct WadSelectPlugin;

#[derive(Component)]
struct CoolButton {
    pub id: i32,
}

#[derive(Component, Default)]
struct ScrollingList {
    position: f32,
}

fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    query_node: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.iter() {
        for (mut scrolling_list, mut style, parent, list_node) in &mut query_list {
            let items_height = list_node.size().y;
            let container_height = query_node.get(parent.get()).unwrap().size().y;

            let max_scroll = container_height / 2.;

            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };

            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, items_height / 4.);
            style.position.top = Val::Px(scrolling_list.position);
        }
    }
}

#[derive(Component)]
struct GUIEl;

fn wad_select_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., 5.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    let base_container = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .insert(GUIEl)
        .id();

    let container = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .insert(GUIEl)
        .id();

    commands.entity(base_container).add_child(container);

    let container2 = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .insert(GUIEl)
        .id();

    commands.entity(base_container).add_child(container2);

    let panel_frame = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::Stretch,
                overflow: Overflow::Hidden,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .insert(GUIEl)
        .id();

    commands.entity(container2).add_child(panel_frame);

    let panel = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::new(Val::Px(10.), Val::Px(10.), Val::Px(10.), Val::Px(0.)),
                ..default()
            },
            ..default()
        })
        .insert(Marker { id: -1 })
        .insert(GUIEl)
        .insert(ScrollingList::default())
        .id();

    commands.entity(panel_frame).add_child(panel);

    let pwad_text = commands
        .spawn(TextBundle {
            text: Text::from_section(
                "No PWAD Selected",
                TextStyle {
                    font: asset_server.load("FiraMono-Medium.ttf"),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            ) // You can still add an alignment.
            .with_alignment(TextAlignment::Center),
            ..default()
        })
        .insert(Marker { id: 1 })
        .insert(GUIEl)
        .id();

    commands.entity(container).add_child(pwad_text);

    let button = commands
        .spawn(ButtonBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(20.),
                    height: Val::Percent(12.),
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .insert(CoolButton { id: 1 })
        .insert(GUIEl)
        .id();

    commands.entity(container).add_child(button);

    let but_text = commands
        .spawn(TextBundle {
            text: Text::from_section(
                "Load PWAD",
                TextStyle {
                    font: asset_server.load("FiraMono-Medium.ttf"),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            ) // You can still add an alignment.
            .with_alignment(TextAlignment::Center),
            ..default()
        })
        .insert(GUIEl)
        .id();

    commands.entity(button).add_child(but_text);

    let iwad_text = commands
        .spawn(TextBundle {
            text: Text::from_section(
                "No IWAD Selected",
                TextStyle {
                    font: asset_server.load("FiraMono-Medium.ttf"),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            ) // You can still add an alignment.
            .with_alignment(TextAlignment::Center),
            ..default()
        })
        .insert(Marker { id: 2 })
        .insert(GUIEl)
        .id();

    commands.entity(container).add_child(iwad_text);

    let button2 = commands
        .spawn(ButtonBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(20.),
                    height: Val::Percent(12.),
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .insert(CoolButton { id: 2 })
        .insert(GUIEl)
        .id();

    commands.entity(container).add_child(button2);

    let but_text2 = commands
        .spawn(TextBundle {
            text: Text::from_section(
                "Load IWAD",
                TextStyle {
                    font: asset_server.load("FiraMono-Medium.ttf"),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            ) // You can still add an alignment.
            .with_alignment(TextAlignment::Center),
            ..default()
        })
        .insert(GUIEl)
        .id();

    commands.entity(button2).add_child(but_text2);

    let spacing = commands
        .spawn(NodeBundle {
            style: Style {
                padding: UiRect {
                    left: Val::Px(0.),
                    right: Val::Px(0.),
                    top: Val::Percent(1.),
                    bottom: Val::Px(0.),
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
}

impl Plugin for WadSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(wad_select_setup.in_schedule(OnEnter(GameState::WadSelect)))
            .add_system(button_system)
            .add_system(mouse_scroll)
            .add_system(cleanup.in_schedule(OnExit(GameState::WadSelect)));
    }
}

fn cleanup(mut commands: Commands, gui_query: Query<Entity, With<GUIEl>>) {
    for entity in gui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn button_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<NextState<GameState>>,
    mut appstate: ResMut<AppState>,
    mut container_query: Query<(Entity, &Marker)>,
    mut text_query: Query<(&mut Text, &Marker)>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &CoolButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();

                info!("cuai!! {}",button.id - 100);

                match button.id {
                    1 => {
                        let _path = FileDialog::new()
                            .set_location("~/Desktop")
                            .add_filter("PWAD File", &["wad"])
                            .show_open_single_file()
                            .unwrap();

                        appstate.pwad_path = _path.clone().unwrap().to_str().unwrap().to_string();

                        for (mut text, marker) in &mut text_query {
                            if (marker.id == 1) {
                                text.sections[0].value = appstate.pwad_path.clone();
                            }
                        }

                        for (gui, marker) in &mut container_query {
                            if marker.id == -1 {
                                let mut pwad = Wad::new();
                                pwad.load_from_file(appstate.pwad_path.clone());

                                for i in 1..9999 {
                                    let formatted_number = format!("{:02}", i);

                                    let map_name = format!("MAP{}", formatted_number);

                                    if pwad.lump(map_name.as_str()).is_some() {
                                        let button = commands
                                            .spawn(ButtonBundle {
                                                style: Style {
                                                    size: Size {
                                                        width: Val::Percent(20.),
                                                        height: Val::Percent(12.),
                                                    },
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    ..default()
                                                },
                                                ..default()
                                            })
                                            .insert(CoolButton { id: 99 + i })
                                            .insert(GUIEl)
                                            .id();

                                        commands.entity(gui).add_child(button);

                                        let but_text = commands
                                            .spawn(TextBundle {
                                                text: Text::from_section(
                                                    map_name,
                                                    TextStyle {
                                                        font: asset_server
                                                            .load("FiraMono-Medium.ttf"),
                                                        font_size: 30.0,
                                                        color: Color::WHITE,
                                                    },
                                                ) // You can still add an alignment.
                                                .with_alignment(TextAlignment::Center),
                                                ..default()
                                            })
                                            .insert(GUIEl)
                                            .id();

                                        commands.entity(button).add_child(but_text);

                                        let spacing = commands
                                            .spawn(NodeBundle {
                                                style: Style {
                                                    padding: UiRect {
                                                        left: Val::Px(0.),
                                                        right: Val::Px(0.),
                                                        top: Val::Percent(1.),
                                                        bottom: Val::Px(0.),
                                                    },
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            })
                                            .id();

                                        commands.entity(gui).add_child(spacing);
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    2 => {
                        let _path = FileDialog::new()
                            .set_location("~/Desktop")
                            .add_filter("IWAD File", &["wad"])
                            .show_open_single_file()
                            .unwrap();

                        appstate.iwad_path = _path.clone().unwrap().to_str().unwrap().to_string();

                        for (mut text, marker) in &mut text_query {
                            if (marker.id == 2) {
                                text.sections[0].value = appstate.iwad_path.clone();
                            }
                        }
                    }
                    100..=9999 => {
                        appstate.map_ind = button.id - 100;
                        state.set(GameState::MapView);
                    }
                    _ => info!("Unknown id"),
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
