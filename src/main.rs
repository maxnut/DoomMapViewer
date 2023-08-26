mod flat;
#[allow(clippy::too_many_arguments)]
mod mapmanager;
mod state;

use std::env;

use bevy::prelude::*;
use state::StatePlugins;

use crate::state::GameState;
use bevy_editor_pls::EditorPlugin;

#[derive(Resource)]
struct AppState {
    iwad_path: String,
    pwad_path: String,
    map_ind: i32
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    App::new()
        .add_state::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin::default())
        .add_startup_system(setup)
        .add_plugins(StatePlugins)
        .run();
}

fn setup(mut commands: Commands) {
    commands.insert_resource(AppState {
        iwad_path: String::new(),
        pwad_path: String::new(),
        map_ind: -1
    });
}
