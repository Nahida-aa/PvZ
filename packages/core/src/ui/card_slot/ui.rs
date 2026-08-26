use bevy::prelude::*;
use bevy::ui::JustifyContent;
use bevy::text::FontSize;
use crate::assets::GameAssets;
use crate::level::LevelDefinition;
use super::components::*;

/// 卡片尺寸
const CARD_WIDTH: f32 = 50.0;
const CARD_HEIGHT: f32 = 70.0;

/// 出战卡槽尺寸
const CARD_SLOT_HEIGHT: f32 = 100.0;

/// 构建选卡界面 UI
pub fn build_card_selection_ui(
    commands: &mut Commands,
    assets: &GameAssets,
    level: &LevelDefinition,
) {
    // 出战卡槽（顶部）
    commands
        .spawn((
            CardSlotRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(-CARD_SLOT_HEIGHT), // 初始隐藏在屏幕外
                width: Val::Percent(100.0),
                height: Val::Px(CARD_SLOT_HEIGHT),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        ))
        .with_children(|parent| {
            // 阳光标签
            parent.spawn((
                SunLabel,
                Text::new("50"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(10.0),
                    top: Val::Px(10.0),
                    ..default()
                },
            ));

            // 卡片占位符
            for i in 0..level.max_choosed_card_num {
                parent.spawn((
                    CardPlaceholder(i as usize),
                    Node {
                        width: Val::Px(CARD_WIDTH),
                        height: Val::Px(CARD_HEIGHT),
                        margin: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    ImageNode::new(assets.seed_packet_silhouette.clone()),
                ));
            }
        });

    // 待选卡面板（底部）
    commands
        .spawn((
            CardCandidatePanel,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                bottom: Val::Px(-600.0), // 初始隐藏在屏幕外
                width: Val::Px(465.0),
                height: Val::Px(500.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
        ))
        .with_children(|parent| {
            // 生成所有可用卡片
            for (i, card_kind) in level.card_kinds.iter().enumerate() {
                let row = i / 8;
                let col = i % 8;

                parent.spawn((
                    CandidateCard,
                    CardEntity {
                        plant_kind: card_kind.clone(),
                        index: i,
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(13.0 + col as f32 * (CARD_WIDTH + 5.0)),
                        top: Val::Px(32.0 + row as f32 * (CARD_HEIGHT + 2.0)),
                        width: Val::Px(CARD_WIDTH),
                        height: Val::Px(CARD_HEIGHT),
                        ..default()
                    },
                    ImageNode::new(assets.card_peashooter.clone()), // TODO: 根据 card_kind 加载对应卡片图片
                ));
            }

            // 开始游戏按钮
            parent.spawn((
                StartGameButton,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::Px(20.0),
                    width: Val::Px(150.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                Text::new("开始游戏"),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
