use bevy::prelude::*;
use serde::Deserialize;

use crate::assets::GameAssets;
use crate::settings::AppConfig;
use crate::level::LevelDefinition;
use crate::lawn::{GridPos, screen_to_world};
use crate::schedule::GameSet;
use crate::state::{GameState, GameplayEntity};
use crate::zombie::Zombie;

#[derive(Component)]
pub struct LawnMower {
    pub row: u32,
    pub running: bool,
}

#[derive(Deserialize)]
struct RigConfig {
    parts: Vec<PartDef>,
}

#[derive(Deserialize)]
struct PartDef {
    image: String,
    x: f32,
    y: f32,
    z: f32,
}

pub struct LawnMowerPlugin;

impl Plugin for LawnMowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_mowers)
            .add_systems(
                Update,
                (mower_detect, mower_run, check_defeat)
                    .in_set(GameSet::Movement)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn mower_image_handle(image: &str, assets: &GameAssets) -> Option<Handle<Image>> {
    match image {
        "LawnMower_body.png" => Some(assets.mower_body.clone()),
        "LawnMower_pull.png" => Some(assets.mower_pull.clone()),
        "LawnMower_engine.png" => Some(assets.mower_engine.clone()),
        "LawnMower_exhaust.png" => Some(assets.mower_exhaust.clone()),
        "Lawnmower_backwheelpiece1.png" | "Lawnmower_backwheelpiece2.png"
        | "Lawnmower_frontwheelpiece1.png" | "Lawnmower_frontwheelpiece2.png" => {
            Some(assets.mower_wheelpiece.clone())
        }
        "Lawnmower_backwheel1.png" | "Lawnmower_backwheel2.png"
        | "Lawnmower_frontwheel2.png" => Some(assets.mower_wheel2.clone()),
        "Lawnmower_frontwheel1.png" => Some(assets.mower_wheel1.clone()),
        "Lawnmower_backwheelshine1.png" | "Lawnmower_backwheelshine2.png"
        | "Lawnmower_wheelshine1.png" | "Lawnmower_wheelshine2.png" => {
            Some(assets.mower_wheelshine.clone())
        }
        _ => None,
    }
}

fn load_rig_config() -> RigConfig {
    // 运行时路径: exe -> ../../assets/items/lawnmower/rig.jsonc
    let exe_path = std::env::current_exe().expect("无法获取可执行文件路径");
    let jsonc_path = exe_path
        .parent()
        .expect("无法获取父目录")
        .join("../../assets/items/lawnmower/rig.jsonc");

    let text = std::fs::read_to_string(&jsonc_path)
        .unwrap_or_else(|e| panic!("读取 {} 失败: {e}", jsonc_path.display()));
    let val: serde_json::Value = jsonc_parser::parse_to_serde_value(&text, &Default::default())
        .expect("解析 rig.jsonc 失败")
        .expect("rig.jsonc 为空");
    serde_json::from_value(val).expect("rig.jsonc 格式错误")
}

fn spawn_mowers(
    mut commands: Commands,
    assets: Res<GameAssets>,
    app: Res<AppConfig>,
    level: Res<LevelDefinition>,
) {
    let config = load_rig_config();

    // 按 z-order 排序
    let mut sorted_parts = config.parts;
    sorted_parts.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());

    // 把割草机的屏幕 X (以视口左上角为原点, 向右为正)
    // 转换成世界坐标 X (以画面中心为原点, 向右为正): 减去半屏宽 win_w/2。
    let mower_x = level.mower.screen_x - app.win_w() / 2.0;
    for row in 0..level.grid.rows {
        let grid_pos = GridPos::new(0, row);
        let ground_y = grid_pos.world_bottom(&level, &app).y;
        commands
            .spawn((
                LawnMower {
                    row,
                    running: false,
                },
                Transform::from_translation(Vec3::new(mower_x, ground_y, 3.0))
                    .with_scale(Vec3::splat(0.8)),
                Visibility::default(),
                GameplayEntity,
            ))
            .with_children(|parent| {
                for part in &sorted_parts {
                    if let Some(handle) = mower_image_handle(&part.image, &assets) {
                        parent.spawn((
                            Sprite::from_image(handle),
                            Transform::from_translation(Vec3::new(part.x, part.y, part.z))
                                .with_scale(Vec3::splat(0.8)),
                        ));
                    }
                }
            });
    }
}

fn mower_detect(
    mut mowers: Query<(Entity, &mut LawnMower, &Transform)>,
    zombies: Query<(&Transform, &GridPos), (With<Zombie>, Without<LawnMower>)>,
    assets: Res<GameAssets>,
    level: Res<LevelDefinition>,
    mut commands: Commands,
) {
    let trigger_range = level.mower.trigger_range;
    for (mower_entity, mut mower, mt) in mowers.iter_mut() {
        if mower.running {
            continue;
        }
        let _ = mower_entity;
        for (zt, zpos) in zombies.iter() {
            if zpos.row == mower.row
                && (zt.translation.x - mt.translation.x).abs() < trigger_range
            {
                mower.running = true;
                commands.spawn((
                    AudioPlayer::<bevy::audio::AudioSource>(assets.lawn_mower_sound.clone()),
                    PlaybackSettings::DESPAWN,
                ));
                break;
            }
        }
    }
}

fn mower_run(
    mut commands: Commands,
    mut mowers: Query<(Entity, &mut LawnMower, &mut Transform)>,
    zombies: Query<(Entity, &Transform, &GridPos), (With<Zombie>, Without<LawnMower>)>,
    level: Res<LevelDefinition>,
) {
    let speed = level.mower.speed;
    let kill_range = level.mower.kill_range;
    for (mower_entity, mower, mut mt) in mowers.iter_mut() {
        if !mower.running {
            continue;
        }
        mt.translation.x += speed;
        for (z_entity, zt, zpos) in zombies.iter() {
            if zpos.row == mower.row && zt.translation.x > mt.translation.x - kill_range {
                commands.entity(z_entity).despawn();
            }
        }
        if mt.translation.x > 530.0 {
            commands.entity(mower_entity).despawn();
        }
    }
}

fn check_defeat(
    zombies: Query<&Transform, (With<Zombie>, Without<LawnMower>)>,
    level: Res<LevelDefinition>,
    app: Res<AppConfig>,
    mut next: ResMut<NextState<GameState>>,
) {
    let defeat_world_x = screen_to_world(level.defeat_screen_x, 0.0, &app).x;
    for zt in zombies.iter() {
        if zt.translation.x < defeat_world_x {
            next.set(GameState::Defeat);
            break;
        }
    }
}
