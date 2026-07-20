pub mod state;
pub mod schedule;
pub mod lawn;
pub mod combat;
pub mod plant;
pub mod zombie;
pub mod projectile;
pub mod input;
pub mod level;
pub mod components;
pub mod sun;
pub mod assets;
pub mod pause_menu;
pub mod animation;
pub mod debug;

use bevy::prelude::*;

use crate::state::GameState;

pub use debug::draw_debug_colliders;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<state::GameState>()
            .configure_sets(
                Update,
                (schedule::GameSet::Spawn, schedule::GameSet::Movement, schedule::GameSet::Combat, schedule::GameSet::Cleanup)
                    .chain(),
            )
            .add_plugins(assets::GameAssetsPlugin)
            .add_plugins(lawn::LawnPlugin)
            .add_plugins(combat::CombatPlugin)
            .add_plugins(plant::PlantPlugin)
            .add_plugins(zombie::ZombiePlugin)
            .add_plugins(projectile::ProjectilePlugin)
            .add_plugins(input::InputPlugin)
            .add_plugins(level::LevelPlugin)
            .add_plugins(components::menebar::GameMenuBarPlugin)
            .add_plugins(sun::SunPlugin)
            .add_plugins(pause_menu::PauseMenuPlugin)
            .add_systems(
                Update,
                animation::animate_sprites
                    .in_set(schedule::GameSet::Movement)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, draw_debug_colliders.in_set(schedule::GameSet::Debug));
    }
}
