use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    Loading,
    #[default]
    Playing,
    Victory,
    Defeat,
}
