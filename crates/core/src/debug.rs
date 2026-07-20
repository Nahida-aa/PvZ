use bevy::prelude::*;

use crate::schedule::GameSet;
use crate::zombie::ZombieCollider;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_debug_colliders.in_set(GameSet::Debug));
    }
}

pub fn draw_debug_colliders(
    mut commands: Commands,
    zombies: Query<(&Transform, &ZombieCollider)>,
) {
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
            Sprite::from_color(color, Vec2::new(width, 1.0)),
            Transform::from_translation(Vec3::new(center.x, min_y, 10.0)),
        ));
        commands.spawn((
            Sprite::from_color(color, Vec2::new(width, 1.0)),
            Transform::from_translation(Vec3::new(center.x, max_y, 10.0)),
        ));
        commands.spawn((
            Sprite::from_color(color, Vec2::new(1.0, height)),
            Transform::from_translation(Vec3::new(min_x, center.y, 10.0)),
        ));
        commands.spawn((
            Sprite::from_color(color, Vec2::new(1.0, height)),
            Transform::from_translation(Vec3::new(max_x, center.y, 10.0)),
        ));
    }
}
