use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    Loading,
    #[default]
    ChoosingCards,
    Playing,
    Paused,
    Victory,
    Defeat,
}

#[derive(Component)]
pub struct GameplayEntity;
