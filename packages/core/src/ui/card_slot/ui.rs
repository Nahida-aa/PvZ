use bevy::prelude::*;
use bevy::text::FontSize;
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
    let panel_w = 465.0;

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
                width: Val::Px(panel_w),
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

            // === 开始游戏按钮 (居中) ===
            let start_btn_w = 156.0;
            let start_btn_h = 42.0;
            let start_btn_x = (panel_w - start_btn_w) / 2.0;
            let start_btn_y = panel_h - start_btn_h - 8.0;
            parent.spawn((
                StartGameButton,
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(start_btn_x),
                    top: Val::Px(start_btn_y),
                    width: Val::Px(start_btn_w),
                    height: Val::Px(start_btn_h),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode::new(assets.seed_chooser_button_glow.clone()),
            )).with_children(|p| {
                p.spawn((
                    Text::new("开始游戏"),
                    TextFont {
                        font_size: FontSize::Px(18.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // === 查看图鉴按钮 (左下) ===
            let btn2_w = 111.0;
            let btn2_h = 26.0;
            let ency_x = 10.0;
            let ency_y = panel_h - btn2_h - 10.0;
            parent.spawn((
                EncyclopediaButton,
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(ency_x),
                    top: Val::Px(ency_y),
                    width: Val::Px(btn2_w),
                    height: Val::Px(btn2_h),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode::new(assets.seed_chooser_button2.clone()),
            )).with_children(|p| {
                p.spawn((
                    Text::new("查看图鉴"),
                    TextFont {
                        font_size: FontSize::Px(14.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // === 重选上次卡片按钮 (右下) ===
            let recard_x = panel_w - btn2_w - 10.0;
            let recard_y = panel_h - btn2_h - 10.0;
            parent.spawn((
                ReCardButton,
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(recard_x),
                    top: Val::Px(recard_y),
                    width: Val::Px(btn2_w),
                    height: Val::Px(btn2_h),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode::new(assets.seed_chooser_button2.clone()),
            )).with_children(|p| {
                p.spawn((
                    Text::new("重选上次卡片"),
                    TextFont {
                        font_size: FontSize::Px(14.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
}
