use bevy::prelude::*;
use crate::assets::GameAssets;
use crate::level::LevelDefinition;
use crate::settings::CardConfig;
use crate::ui::card_visual;
use super::components::*;

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
    _level: &LevelDefinition,
    cc: &CardConfig,
) {
    let cols = cc.candidate_cols;
    let rows = cc.candidate_rows;
    let total = cols * rows;
    let panel_h = 513.0;

    let mut cards_info: Vec<(usize, &str, f32, f32)> = Vec::new();
    for i in 0..total {
        let col = i % cols;
        let row = i / cols;
        let x = cc.candidate_offset_x + col as f32 * (cc.card_slot_width + cc.candidate_card_gap_x);
        let y = cc.candidate_offset_y + row as f32 * (cc.card_slot_height + cc.candidate_card_gap_y);
        if i < ALL_PLANTS.len() {
            cards_info.push((i, ALL_PLANTS[i], x, y));
        }
    }

    commands
        .spawn((
            CardCandidatePanel,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(150.0),
                top: Val::Px(87.0 + panel_h),
                width: Val::Px(465.0),
                height: Val::Px(panel_h),
                ..default()
            },
            ImageNode::new(assets.seed_chooser_background.clone()),
        ))
        .with_children(|parent| {
            for &(i, plant, x, y) in &cards_info {
                card_visual::spawn_card(parent, plant, assets, cc, x, y)
                    .insert((
                        CandidateCard,
                        CardEntity {
                            plant_kind: plant.to_string(),
                            index: i,
                        },
                    ));
            }

            for i in cards_info.len()..total {
                let col = i % cols;
                let row = i / cols;
                let x = cc.candidate_offset_x + col as f32 * (cc.card_slot_width + cc.candidate_card_gap_x);
                let y = cc.candidate_offset_y + row as f32 * (cc.card_slot_height + cc.candidate_card_gap_y);
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(x),
                        top: Val::Px(y),
                        width: Val::Px(cc.card_slot_width),
                        height: Val::Px(cc.card_slot_height),
                        ..default()
                    },
                    ImageNode::new(assets.seed_packet_silhouette.clone()),
                ));
            }
        });
}
