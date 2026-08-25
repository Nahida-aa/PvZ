use bevy::prelude::*;

use crate::config::{AppConfig, LevelDefinition};
use crate::lawn_mower::LawnMower;
use crate::schedule::GameSet;
use crate::zombie::ZombieCollider;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_debug_colliders.in_set(GameSet::Debug));
    }
}

#[derive(Component)]
pub struct DebugLine;

pub fn draw_debug_colliders(
    mut commands: Commands,
    zombies: Query<(&Transform, &ZombieCollider)>,
    mowers: Query<&Transform, With<LawnMower>>,
    old_lines: Query<Entity, With<DebugLine>>,
    app: Res<AppConfig>,
    level: Res<LevelDefinition>,
) {
    let win_w = app.win_w();
    let win_h = app.win_h();
    for line in old_lines.iter() {
        commands.entity(line).despawn();
    }

    let axis_color = Color::srgba(0.0, 1.0, 1.0, 0.8);
    commands.spawn((
        DebugLine,
        Sprite::from_color(axis_color, Vec2::new(1.0, win_h)),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
    ));
    let edge_color = Color::srgba(1.0, 1.0, 0.0, 0.4);
    for &x in &[-win_w / 2.0, win_w / 2.0] {
        commands.spawn((
            DebugLine,
            Sprite::from_color(edge_color, Vec2::new(1.0, win_h)),
            Transform::from_translation(Vec3::new(x, 0.0, 10.0)),
        ));
    }

    for (transform, collider) in zombies.iter() {
        let origin = transform.translation.truncate();
        for (i, rect) in collider.rects.iter().enumerate() {
            let center = origin + rect.center_offset;
            let half = rect.half_size;

            let min_x = center.x - half.x;
            let max_x = center.x + half.x;
            let min_y = center.y - half.y;
            let max_y = center.y + half.y;

            let color = if i == 0 {
                Color::srgba(1.0, 0.0, 0.0, 0.3)
            } else {
                Color::srgba(1.0, 0.5, 0.0, 0.3)
            };

            let width = max_x - min_x;
            let height = max_y - min_y;

            commands.spawn((
                DebugLine,
                Sprite::from_color(color, Vec2::new(width, 1.0)),
                Transform::from_translation(Vec3::new(center.x, min_y, 10.0)),
            ));
            commands.spawn((
                DebugLine,
                Sprite::from_color(color, Vec2::new(width, 1.0)),
                Transform::from_translation(Vec3::new(center.x, max_y, 10.0)),
            ));
            commands.spawn((
                DebugLine,
                Sprite::from_color(color, Vec2::new(1.0, height)),
                Transform::from_translation(Vec3::new(min_x, center.y, 10.0)),
            ));
            commands.spawn((
                DebugLine,
                Sprite::from_color(color, Vec2::new(1.0, height)),
                Transform::from_translation(Vec3::new(max_x, center.y, 10.0)),
            ));
        }
    }

    // 割草机碰撞箱 (对应 Godot RectangleShape2D 33.5x48, 位于车身上方)
    let mower_color = Color::srgba(1.0, 1.0, 1.0, 0.5);
    let mower_world_x = level.mower.screen_x - win_w / 2.0;
    for mt in mowers.iter() {
        let cy = mt.translation.y + level.mower.y_offset;
        let half_w = level.mower.hit_w / 2.0;
        let half_h = level.mower.hit_h / 2.0;
        let min_x = mower_world_x - half_w;
        let max_x = mower_world_x + half_w;
        let min_y = cy - half_h;
        let max_y = cy + half_h;
        commands.spawn((
            DebugLine,
            Sprite::from_color(mower_color, Vec2::new(level.mower.hit_w, 1.0)),
            Transform::from_translation(Vec3::new(mower_world_x, min_y, 10.0)),
        ));
        commands.spawn((
            DebugLine,
            Sprite::from_color(mower_color, Vec2::new(level.mower.hit_w, 1.0)),
            Transform::from_translation(Vec3::new(mower_world_x, max_y, 10.0)),
        ));
        commands.spawn((
            DebugLine,
            Sprite::from_color(mower_color, Vec2::new(1.0, level.mower.hit_h)),
            Transform::from_translation(Vec3::new(min_x, cy, 10.0)),
        ));
        commands.spawn((
            DebugLine,
            Sprite::from_color(mower_color, Vec2::new(1.0, level.mower.hit_h)),
            Transform::from_translation(Vec3::new(max_x, cy, 10.0)),
        ));
    }

    // 房子碰撞箱 (对应 Godot Area2DHome 的 RectangleShape2D 150x600, 覆盖全高)
    let house_color = Color::srgba(1.0, 0.0, 1.0, 0.4);
    let house_world_x = level.defeat_screen_x - win_w / 2.0;
    let house_h = win_h;
    commands.spawn((
        DebugLine,
        Sprite::from_color(house_color, Vec2::new(150.0, 1.0)),
        Transform::from_translation(Vec3::new(house_world_x, win_h / 2.0, 10.0)),
    ));
    commands.spawn((
        DebugLine,
        Sprite::from_color(house_color, Vec2::new(150.0, 1.0)),
        Transform::from_translation(Vec3::new(house_world_x, -win_h / 2.0, 10.0)),
    ));
    commands.spawn((
        DebugLine,
        Sprite::from_color(house_color, Vec2::new(1.0, house_h)),
        Transform::from_translation(Vec3::new(house_world_x - 75.0, 0.0, 10.0)),
    ));
    commands.spawn((
        DebugLine,
        Sprite::from_color(house_color, Vec2::new(1.0, house_h)),
        Transform::from_translation(Vec3::new(house_world_x + 75.0, 0.0, 10.0)),
    ));
}
