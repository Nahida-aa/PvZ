use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use crate::assets::GameAssets;
use crate::components::menebar::SunBank;
use crate::lawn::{GRID_COLS, GRID_ROWS, GridPos, WIN_W, screen_to_world};
use crate::schedule::GameSet;
use crate::state::GameState;

/// 阳光 存活 时间
const SUN_LIVE_TIME: f32 = 10.0;
/// 系统 阳光 生成 间隔 时间
const SYSTEM_SUN_INTERVAL_BASE: f32 = 4.25;

#[derive(Component)]
pub struct Sun {
    pub dest_world_x: f32,
    pub dest_world_y: f32,
    pub speed: f32,
    pub alive_timer: f32,
}

#[derive(Component)]
struct SystemSunTimer {
    timer: Timer,
}

pub struct SunPlugin;

impl Plugin for SunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SunBank>()
            .add_systems(OnEnter(GameState::Playing), spawn_system_sun_timer)
            .add_systems(Update, system_sun_tick.in_set(GameSet::Spawn))
            .add_systems(Update, sun_move.in_set(GameSet::Movement))
            .add_systems(Update, collect_sun.in_set(GameSet::Cleanup));
    }
}

fn spawn_system_sun_timer(mut commands: Commands) {
    commands.spawn(SystemSunTimer {
        timer: Timer::from_seconds(SYSTEM_SUN_INTERVAL_BASE, TimerMode::Repeating),
    });
}

fn system_sun_tick(
    time: Res<Time>,
    mut timers: Query<&mut SystemSunTimer>,
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    for mut timer_entity in timers.iter_mut() {
        timer_entity.timer.tick(time.delta());
        if timer_entity.timer.just_finished() {
            let mut rng = rand::thread_rng();
            let row = rng.gen_range(0..GRID_ROWS as usize);
            let col = rng.gen_range(0..GRID_COLS as usize);
            let grid_pos = GridPos::new(col as u32, row as u32);
            let dest = grid_pos.world_bottom();

            let mut rng = rand::thread_rng();
            let start_screen_x = WIN_W / 2.0 - (WIN_W / 2.0 - 100.0) + rng.gen_range(0.0..200.0);
            let start_screen_y = 10.0;
            let start_world = screen_to_world(start_screen_x, start_screen_y);

            let frames = assets.sun_frames.clone();
            commands.spawn((
                Sun {
                    dest_world_x: dest.x,
                    dest_world_y: dest.y,
                    speed: 40.0,
                    alive_timer: 0.0,
                },
                Sprite::from_image(frames[0].clone()),
                Transform::from_translation(Vec3::new(start_world.x, start_world.y, 3.0)),
                Visibility::default(),
            ));
        }
    }
}

fn sun_move(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Sun, Entity)>,
    mut commands: Commands,
) {
    for (mut transform, mut sun, entity) in query.iter_mut() {
        let dx = sun.dest_world_x - transform.translation.x;
        let dy = sun.dest_world_y - transform.translation.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist > 1.0 {
            transform.translation.x += (dx / dist) * sun.speed * time.delta_secs();
            transform.translation.y += (dy / dist) * sun.speed * time.delta_secs();
        } else {
            sun.alive_timer += time.delta_secs();
            if sun.alive_timer > SUN_LIVE_TIME {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn collect_sun(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<(Entity, &Transform, &Sun)>,
    mut bank: ResMut<SunBank>,
    mut commands: Commands,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let Ok(window) = window.single() else {
        return;
    };
    let mouse_position = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, mouse_position) else {
        return;
    };
    let mouse_world = ray.origin;

    for (entity, transform, _sun) in query.iter_mut() {
        let dx = mouse_world.x - transform.translation.x;
        let dy = mouse_world.y - transform.translation.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 30.0 {
            bank.amount += 25;
            commands.entity(entity).despawn();
        }
    }
}
