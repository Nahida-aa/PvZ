use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::ui::ZIndex;
use bevy::window::PrimaryWindow;

use crate::assets::GameAssets;
use crate::lawn::GridPos;
use crate::plant::{PlantKind, SpawnPlant};
use crate::state::GameState;
use crate::components::menebar::SunBank;
use crate::components::plant_cards::PlantCards;

#[derive(Resource, Default)]
pub struct SelectedPlant {
    pub kind: Option<PlantKind>,
}

#[derive(Component)]
struct PlantGhost;

#[derive(Component)]
struct PlantGhostHint;

const GHOST_W: f32 = 73.0;
const GHOST_H: f32 = 74.0;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedPlant>()
            .add_systems(
                Update,
                (
                    update_ghost,
                    handle_click_to_place,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn plant_texture(kind: PlantKind, assets: &GameAssets) -> Handle<Image> {
    match kind {
        PlantKind::Peashooter => assets.peashooter_frames[0].clone(),
        PlantKind::Sunflower => assets.sunflower_frames[0].clone(),
    }
}

fn update_ghost(
    mut commands: Commands,
    selected: Res<SelectedPlant>,
    assets: Res<GameAssets>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    ghost: Query<(Entity, &PlantGhost)>,
    hint: Query<(Entity, &PlantGhostHint)>,
) {
    match selected.kind {
        None => {
            for (entity, _) in ghost.iter() {
                commands.entity(entity).despawn();
            }
            for (entity, _) in hint.iter() {
                commands.entity(entity).despawn();
            }
        }
        Some(kind) => {
            let cursor = window
                .single()
                .ok()
                .and_then(|w| w.cursor_position());
            let world_pos = cursor.and_then(|c| {
                camera
                    .single()
                    .ok()
                    .and_then(|(cam, cam_t)| cam.viewport_to_world_2d(cam_t, c).ok())
            });

            let snapped = world_pos
                .and_then(|p| GridPos::from_world(p))
                .map(|g| g.world_bottom());

            // 跟手影子（UI 层，满透明，不被工具栏遮挡）
            match (ghost.single(), cursor) {
                (Ok((entity, _)), Some(c)) => {
                    commands.entity(entity).insert(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(c.x - GHOST_W / 2.0),
                        top: Val::Px(c.y - GHOST_H),
                        ..default()
                    });
                }
                (Ok((entity, _)), None) => {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
                (Err(_), Some(c)) => {
                    commands.spawn((
                        PlantGhost,
                        ImageNode::new(plant_texture(kind, &assets)),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(c.x - GHOST_W / 2.0),
                            top: Val::Px(c.y - GHOST_H),
                            ..default()
                        },
                        ZIndex(100),
                        Visibility::default(),
                    ));
                }
                (Err(_), None) => {}
            }

            // 格子吸附预览（半透明，仅在有效格子内显示）
            match (hint.single(), snapped) {
                (Ok((entity, _)), Some(cell)) => {
                    commands.entity(entity).insert((
                        Transform::from_translation(Vec3::new(cell.x, cell.y, 5.0)),
                        Visibility::default(),
                    ));
                }
                (Ok((entity, _)), None) => {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
                (Err(_), Some(cell)) => {
                    commands.spawn((
                        PlantGhostHint,
                        Sprite {
                            image: plant_texture(kind, &assets),
                            color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                            ..default()
                        },
                        Anchor::BOTTOM_CENTER,
                        Transform::from_translation(Vec3::new(cell.x, cell.y, 5.0)),
                        Visibility::default(),
                    ));
                }
                (Err(_), None) => {}
            }
        }
    }
}

fn handle_click_to_place(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut spawner: MessageWriter<SpawnPlant>,
    mut selected: ResMut<SelectedPlant>,
    mut bank: ResMut<SunBank>,
    mut cards: ResMut<PlantCards>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        selected.kind = None;
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(plant) = selected.kind else {
        return;
    };
    if bank.amount < plant.cost() {
        return;
    }
    if !cards.ready(&plant) {
        return;
    }
    let Ok(window) = window.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        return;
    };
    let Some(grid_pos) = GridPos::from_world(world_pos) else {
        return;
    };
    bank.amount -= plant.cost();
    spawner.write(SpawnPlant { kind: plant, pos: grid_pos });
    cards.trigger(&plant);
    selected.kind = None;
}
