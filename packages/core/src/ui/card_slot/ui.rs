use bevy::prelude::*;
use crate::assets::GameAssets;
use crate::level::LevelDefinition;
use super::components::*;

const CARD_W: f32 = 50.0;
const CARD_H: f32 = 70.0;
const COLS: usize = 8;

/// 对齐 menubar 的 bar_left
const PANEL_LEFT: f32 = 150.0;
/// menubar 高度 = SeedBank.png 高 87px
const PANEL_TOP: f32 = 87.0;

/// 所有可用植物卡片（按卡片文件名顺序）
const ALL_PLANTS: &[&str] = &[
    "Peashooter", "Sunflower", "CherryBomb", "WallNut",
    "PotatoMine", "SnowPea", "Chomper", "RepeaterPea",
    "PuffShroom", "SunShroom", "FumeShroom", "GraveBuster",
    "HypnoShroom", "IceShroom", "DoomShroom", "LilyPad",
    "Squash", "ThreePeaShooter", "TangleKlep", "Jalapeno",
    "Spikeweed", "TorchWood", "TallNut", "Seashroom",
    "Plantern", "Cactus", "Blover", "StarFruit",
    "PumpkinHead", "Garlic", "GiantWallNut",
];

pub fn build_card_selection_ui(
    commands: &mut Commands,
    assets: &GameAssets,
    level: &LevelDefinition,
) {
    let total_slots = ALL_PLANTS.len();
    let rows = (total_slots + COLS - 1) / COLS;

    commands
        .spawn((
            CardCandidatePanel,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(PANEL_LEFT),
                top: Val::Px(PANEL_TOP + 513.0), // 初始隐藏
                width: Val::Px(465.0),
                height: Val::Px(513.0),
                ..default()
            },
            ImageNode::new(assets.seed_chooser_background.clone()),
        ))
        .with_children(|parent| {
            for i in 0..(rows * COLS) {
                let col = i % COLS;
                let row = i / COLS;
                let x = 13.0 + col as f32 * (CARD_W + 5.0);
                let y = 32.0 + row as f32 * (CARD_H + 2.0);

                if i < total_slots {
                    // 有卡片的位置
                    let plant = ALL_PLANTS[i];
                    let handle = card_handle(plant, assets);
                    parent.spawn((
                        CandidateCard,
                        CardEntity {
                            plant_kind: plant.to_string(),
                            index: i,
                        },
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(x),
                            top: Val::Px(y),
                            width: Val::Px(CARD_W),
                            height: Val::Px(CARD_H),
                            ..default()
                        },
                        ImageNode::new(handle),
                    ));
                } else {
                    // 空位补轮廓
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(x),
                            top: Val::Px(y),
                            width: Val::Px(CARD_W),
                            height: Val::Px(CARD_H),
                            ..default()
                        },
                        ImageNode::new(assets.seed_packet_silhouette.clone()),
                    ));
                }
            }
        });
}

fn card_handle(plant: &str, assets: &GameAssets) -> Handle<Image> {
    match plant {
        "Peashooter" => assets.card_peashooter.clone(),
        "Sunflower" => assets.card_sunflower.clone(),
        _ => assets.card_peashooter.clone(), // TODO: 加载其他植物卡片图片
    }
}
