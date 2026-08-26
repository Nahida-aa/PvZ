use bevy::prelude::*;
use crate::assets::GameAssets;
use crate::level::LevelDefinition;
use crate::ui::card_visual;
use super::components::*;

const CARD_W: f32 = 50.0;
const CARD_H: f32 = 70.0;
const COLS: usize = 8;
const ROWS: usize = 6;

const PANEL_LEFT: f32 = 150.0;
const PANEL_TOP: f32 = 87.0;

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
    let params = card_visual::CardVisualParams {
        width: CARD_W,
        height: CARD_H,
        font: assets.font.clone(),
    };

    // 先收集要生成的卡片信息，避免在 with_children 里借用 commands
    let mut cards_info: Vec<(usize, &str, f32, f32)> = Vec::new();
    for i in 0..(ROWS * COLS) {
        let col = i % COLS;
        let row = i / COLS;
        let x = 13.0 + col as f32 * (CARD_W + 5.0);
        let y = 32.0 + row as f32 * (CARD_H + 2.0);
        if i < ALL_PLANTS.len() {
            cards_info.push((i, ALL_PLANTS[i], x, y));
        }
    }

    commands
        .spawn((
            CardCandidatePanel,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(PANEL_LEFT),
                top: Val::Px(PANEL_TOP + 513.0),
                width: Val::Px(465.0),
                height: Val::Px(513.0),
                ..default()
            },
            ImageNode::new(assets.seed_chooser_background.clone()),
        ))
        .with_children(|parent| {
            // 生成所有卡片
            for &(i, plant, x, y) in &cards_info {
                card_visual::spawn_card(parent, plant, assets, &params, x, y)
                    .insert((
                        CandidateCard,
                        CardEntity {
                            plant_kind: plant.to_string(),
                            index: i,
                        },
                    ));
            }

            // 空位补轮廓
            for i in cards_info.len()..(ROWS * COLS) {
                let col = i % COLS;
                let row = i / COLS;
                let x = 13.0 + col as f32 * (CARD_W + 5.0);
                let y = 32.0 + row as f32 * (CARD_H + 2.0);
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
        });
}
