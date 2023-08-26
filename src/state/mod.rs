mod wad_select;
mod map_view;

use crate::state::wad_select::WadSelectPlugin;
use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::prelude::{Component, States};

use self::map_view::MapViewPlugin;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    WadSelect,
    MapView,
}

#[derive(Component)]
pub struct Marker {
    id: i32,
}

pub struct StatePlugins;

impl PluginGroup for StatePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(WadSelectPlugin).add(MapViewPlugin)
    }
}
