use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::animation::SpriteAnimation;
use crate::assets::GameAssets;
use crate::combat::{DeathCleanup, Health, Team};
use crate::lawn::{CELL_WIDTH, GridPos, WIN_W};
use crate::schedule::GameSet;
use crate::state::GameState;

#[derive(Component)]
pub struct Zombie {
    pub name: &'static str,
    pub speed: f32,
}

#[derive(Component)]
pub struct Walker {
    pub base_speed: f32,
}

#[derive(Clone, Copy)]
pub struct ColliderRect {
    pub half_size: Vec2,
    pub center_offset: Vec2,
}

#[derive(Component)]
pub struct ZombieCollider {
    pub rects: Vec<ColliderRect>,
}

#[derive(Message)]
pub struct SpawnZombie {
    pub row: u32,
}

pub struct ZombiePlugin;

impl Plugin for ZombiePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnZombie>()
            .add_systems(
                Update,
                (
                    handle_spawn_zombie.in_set(GameSet::Spawn),
                    zombie_walk.in_set(GameSet::Movement),
                    cleanup_offscreen_zombies.in_set(GameSet::Cleanup),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn handle_spawn_zombie(
    mut events: MessageReader<SpawnZombie>,
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    for ev in events.read() {
        let grid_pos = GridPos::new(crate::lawn::GRID_COLS - 1, ev.row);
        let start_x = 830.0 - WIN_W / 2.0;
        let frames = assets.normal_zombie_frames.clone();
        commands.spawn((
            Zombie {
                name: "Basic",
                speed: 0.3,
            },
            Health::new(200.0),
            Team::Zombie,
            Walker { base_speed: 0.3 },
                ZombieCollider {
                    rects: vec![
                        ColliderRect {
                            half_size: Vec2::new(20.0, 55.0),
                            center_offset: Vec2::new(105.0, 60.0),
                        },
                        ColliderRect {
                            half_size: Vec2::new(28.0, 18.0),
                            center_offset: Vec2::new(95.0, 18.0),
                        },
                    ],
                },
            DeathCleanup,
            crate::state::GameplayEntity,
            grid_pos,
            Sprite::from_image(frames[0].clone()),
            Anchor::BOTTOM_LEFT,
            SpriteAnimation {
                frames,
                frame_duration: 0.1,
                timer: 0.0,
                current: 0,
            },
            Transform::from_translation(Vec3::new(start_x, grid_pos.world_bottom().y, 2.0)),
            Visibility::default(),
        ));
    }
}

fn zombie_walk(mut query: Query<(&mut Transform, &Walker)>) {
    for (mut transform, walker) in query.iter_mut() {
        transform.translation.x -= walker.base_speed;
    }
}

fn cleanup_offscreen_zombies(
    mut commands: Commands,
    zombies: Query<(Entity, &Transform), With<Zombie>>,
) {
    let threshold = -WIN_W / 2.0 - CELL_WIDTH;
    for (entity, transform) in zombies.iter() {
        if transform.translation.x < threshold {
            commands.entity(entity).despawn();
        }
    }
}
