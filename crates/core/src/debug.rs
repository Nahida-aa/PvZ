use bevy::prelude::*;

use crate::lawn::{WIN_H, WIN_W};
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
    old_lines: Query<Entity, With<DebugLine>>,
) {
    for line in old_lines.iter() {
        commands.entity(line).despawn();
    }

    let axis_color = Color::srgba(0.0, 1.0, 1.0, 0.8);
    commands.spawn((
        DebugLine,
        Sprite::from_color(axis_color, Vec2::new(1.0, WIN_H)),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
    ));
    let edge_color = Color::srgba(1.0, 1.0, 0.0, 0.4);
    for &x in &[-WIN_W / 2.0, WIN_W / 2.0] {
        commands.spawn((
            DebugLine,
            Sprite::from_color(edge_color, Vec2::new(1.0, WIN_H)),
            Transform::from_translation(Vec3::new(x, 0.0, 10.0)),
        ));
    }

    for (transform, collider) in zombies.iter() {
        let center = transform.translation.truncate() + collider.center_offset;
        let half = collider.half_size;

        let min_x = center.x - half.x;
        let max_x = center.x + half.x;
        let min_y = center.y - half.y;
        let max_y = center.y + half.y;

        let color = Color::srgba(1.0, 0.0, 0.0, 0.3);

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
