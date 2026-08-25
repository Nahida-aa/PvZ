pub mod end_screen;
pub mod pause_menu;

use bevy::prelude::*;

/// 递归销毁实体及其所有子实体。
pub(crate) fn despawn_recursive(
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
) {
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            despawn_recursive(commands, child, children_query);
        }
    }
    commands.entity(entity).despawn();
}
