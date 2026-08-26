use bevy::prelude::*;
use bevy::text::FontSize;
use crate::assets::GameAssets;
use crate::level::LevelDefinition;
use super::components::*;

const CARD_W: f32 = 50.0;
const CARD_H: f32 = 70.0;

pub fn build_card_selection_ui(
    commands: &mut Commands,
    assets: &GameAssets,
    level: &LevelDefinition,
) {
    // ── 出战卡槽（SeedBank.png 背景，446×87）──
    commands
        .spawn((
            CardSlotRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(-87.0), // 初始隐藏
                width: Val::Px(446.0),
                height: Val::Px(87.0),
                ..default()
            },
            ImageNode::new(assets.seed_bank.clone()),
        ))
        .with_children(|parent| {
            for i in 0..level.max_choosed_card_num {
                parent.spawn((
                    CardPlaceholder(i as usize),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(75.0 + i as f32 * (CARD_W + 1.0)),
                        top: Val::Px(8.0),
                        width: Val::Px(CARD_W),
                        height: Val::Px(CARD_H),
                        ..default()
                    },
                    ImageNode::new(assets.seed_packet_silhouette.clone()),
                ));
            }
        });

    // ── 待选卡面板（SeedChooser_Background.png 背景，465×513）──
    commands
        .spawn((
            CardCandidatePanel,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(-513.0), // 初始隐藏在屏幕外
                width: Val::Px(465.0),
                height: Val::Px(513.0),
                ..default()
            },
            ImageNode::new(assets.seed_chooser_background.clone()),
        ))
        .with_children(|parent| {
            // 卡片网格（8列，间距5px）
            for (i, card_kind) in level.card_kinds.iter().enumerate() {
                let col = i % 8;
                let row = i / 8;
                let handle = match card_kind.as_str() {
                    "Peashooter" => assets.card_peashooter.clone(),
                    "Sunflower" => assets.card_sunflower.clone(),
                    _ => assets.card_peashooter.clone(),
                };
                parent.spawn((
                    CandidateCard,
                    CardEntity {
                        plant_kind: card_kind.clone(),
                        index: i,
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(13.0 + col as f32 * (CARD_W + 5.0)),
                        top: Val::Px(32.0 + row as f32 * (CARD_H + 2.0)),
                        width: Val::Px(CARD_W),
                        height: Val::Px(CARD_H),
                        ..default()
                    },
                    ImageNode::new(handle),
                ));
            }

            // 开始游戏按钮（居中偏下）
            parent.spawn((
                StartGameButton,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(150.0),
                    bottom: Val::Px(50.0),
                    width: Val::Px(165.0),
                    height: Val::Px(42.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode::new(assets.start_button_glow.clone()),
                Text::new("开始游戏"),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
