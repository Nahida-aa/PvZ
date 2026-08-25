use bevy::prelude::*;

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

fn spawn_mowers(
    mut commands: Commands,
    assets: Res<GameAssets>,
    app: Res<AppConfig>,
    level: Res<LevelDefinition>,
) {
    // 部件布局对应 Godot lawn_mower.tscn。
    // Godot 变换链: 根 -> Body(position (-39,-55), scale 0.8) -> 部件(position 相对 Body, scale 0.8)
    // 部件在 Godot 中 centered=false, 故 position 为左上角; 转 Bevy 中心锚点需补 size*0.8/2。
    // 中心相对根(Godot y-down) = (-39 + 0.8*px + 0.8*sx/2, -55 + 0.8*py + 0.8*sy/2)
    // Bevy y 向上, 故 y 取负。每个子 Sprite 再乘自身 scale 0.8 -> 总缩放 0.64。
    // z 按 Godot 声明顺序: 后轮(底) < 车身 < 前轮/引擎/排气(顶), 保证车身被其它部件遮挡。
    let parts: &[(Handle<Image>, f32, f32, f32)] = &[
        (assets.mower_body.clone(), -13.0, 27.16, 5.0),
        (assets.mower_wheelpiece.clone(), -7.56, 23.72, 1.0),
        (assets.mower_wheel2.clone(), -8.2, 21.4, 2.0),
        (assets.mower_wheelshine.clone(), -8.6, 21.88, 3.0),
        (assets.mower_wheelpiece.clone(), 19.0, 23.64, 1.0),
        (assets.mower_wheel2.clone(), 18.28, 21.24, 2.0),
        (assets.mower_wheelshine.clone(), 18.2, 21.72, 3.0),
        (assets.mower_pull.clone(), -36.12, 16.2, 8.0),
        (assets.mower_engine.clone(), 7.0, 19.64, 9.0),
        (assets.mower_wheelpiece.clone(), -12.44, 8.12, 6.0),
        (assets.mower_wheel1.clone(), -13.48, 5.4, 7.0),
        (assets.mower_wheelshine.clone(), -14.04, 5.56, 8.0),
        (assets.mower_wheelpiece.clone(), 15.4, 7.0, 6.0),
        (assets.mower_wheel2.clone(), 14.68, 4.92, 7.0),
        (assets.mower_wheelshine.clone(), 13.88, 5.08, 8.0),
        (assets.mower_exhaust.clone(), -4.44, 12.44, 10.0),
    ];

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
                for (tex, ox, oy, z) in parts.iter() {
                    parent.spawn((
                        Sprite::from_image(tex.clone()),
                        Transform::from_translation(Vec3::new(*ox, *oy, *z))
                            .with_scale(Vec3::splat(0.8)),
                    ));
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
