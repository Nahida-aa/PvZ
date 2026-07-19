use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Spawn,
    Movement,
    Combat,
    Cleanup,
}
