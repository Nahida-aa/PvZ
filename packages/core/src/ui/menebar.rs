use bevy::prelude::*;
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
use bevy::ui::prelude::{BorderRect, NodeImageMode, SliceScaleMode, TextureSlicer};
use bevy::ui::ZIndex;

use crate::assets::GameAssets;
use crate::config::LevelDefinition;
use crate::plant::PlantKind;
use crate::state::GameState;
use crate::ui::plant_cards::PlantCards;

#[derive(Resource)]
pub struct SunBank {
    pub amount: u32,
}

impl Default for SunBank {
    fn default() -> Self {
        Self { amount: 150 }
    }
}

#[derive(Component)]
struct SunCounter;

/// 阳光收集目标点：阳光飘向菜单栏时的终点位置。
#[derive(Component)]
pub struct SunTarget;

#[derive(Component)]
pub struct PlantCard {
    pub kind: PlantKind,
    pub cooldown_timer: f32,
    pub cooldown_duration: f32,
}

#[derive(Component)]
pub struct CardCooldownOverlay;

#[derive(Component)]
pub struct CardSelectedOverlay;

impl Default for PlantCard {
    fn default() -> Self {
        Self {
            kind: PlantKind::Peashooter,
            cooldown_timer: 0.0,
            cooldown_duration: 0.0,
        }
    }
}

pub struct GameMenuBarPlugin;

impl Plugin for GameMenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SunBank>()
            .init_resource::<PlantCards>()
            .add_systems(OnEnter(GameState::Playing), setup_menubar)
            .add_systems(
                Update,
                (update_sun_counter, handle_card_click, sync_selected_overlay, cooldown_tick)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn card_image(kind: PlantKind, assets: &GameAssets) -> Handle<Image> {
    match kind {
        PlantKind::Peashooter => assets.card_peashooter.clone(),
        PlantKind::Sunflower => assets.card_sunflower.clone(),
    }
}

fn setup_menubar(
    mut commands: Commands,
    assets: Res<GameAssets>,
    level: Res<LevelDefinition>,
    mut cards: ResMut<PlantCards>,
) {
    let font = assets.font.clone();
    let sun_font = assets.sun_font.clone();
    // 背景宽度 = 阳光区(78) + 卡槽数 × 50 + 右边距(12)
    let bg_width = 78.0 + level.max_choosed_card_num as f32 * 50.0 + 12.0;
    commands
        .spawn((
            // 顶部菜单栏背景：SeedBank.png (原始 446×87)
            // 对齐 Godot CardSlotBattle 的 StyleBoxTexture 九宫格切分：
            // texture_margin = (78, 10, 12, 10)
            // 中间区域根据卡槽数动态拉伸
            Node {
                width: Val::Px(bg_width),
                height: Val::Px(87.0),
                position_type: PositionType::Absolute,
                left: Val::Px(150.0),
                top: Val::Px(0.0),
                ..default()
            },
            ImageNode {
                image: assets.seed_bank.clone(),
                image_mode: NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect {
                        min_inset: Vec2::new(78.0, 10.0),
                        max_inset: Vec2::new(12.0, 10.0),
                    },
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 100.0,
                }),
                ..default()
            },
            ZIndex(-1),
            crate::state::GameplayEntity,
        ))
        .with_children(|parent| {
            // 阳光数量文字（对齐 Godot CurrSunValue: font_size=20, color=black）
            parent.spawn((
                SunCounter,
                Text::new("150"),
                TextFont {
                    font: FontSource::Handle(sun_font.clone()),
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::srgb(0.0, 0.0, 0.0)),
                BackgroundColor(Color::NONE),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(10.0),
                    bottom: Val::Px(6.0),
                    width: Val::Px(58.0),
                    height: Val::Px(28.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ));

            // 阳光收集目标点（对齐 Godot Marker2DSunTarget: position=(-39, 26)）
            parent.spawn((
                SunTarget,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(39.0),
                    top: Val::Px(26.0),
                    width: Val::Px(0.0),
                    height: Val::Px(0.0),
                    ..default()
                },
            ));

            // 从关卡配置解析出战卡牌
            let slot_kinds: Vec<PlantKind> = level
                .card_kinds
                .iter()
                .filter_map(|s| PlantKind::from_str(s))
                .collect();
            cards.init(&slot_kinds);

            let max_slots = level.max_choosed_card_num;

            // 先按 max_choosed_card_num 创建所有槽位，有卡牌的再覆盖
            for i in 0..max_slots {
                let x = 78.0 + i as f32 * 50.0;
                let has_card = (i as usize) < slot_kinds.len();
                let kind = if has_card { slot_kinds[i as usize] } else { PlantKind::Peashooter };

                let mut entity_cmds = parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(x),
                        top: Val::Px(8.0),
                        width: Val::Px(50.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::End,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ));

                // 有卡牌的槽位加 Button + PlantCard 组件
                if has_card {
                    entity_cmds.insert((
                        Button,
                        PlantCard {
                            kind,
                            cooldown_timer: 0.0,
                            cooldown_duration: crate::ui::plant_cards::cooldown_duration(kind),
                        },
                    ));
                }

                entity_cmds.with_children(|parent| {
                    // 所有槽位都有占位图背景
                    parent.spawn((
                        ImageNode::new(assets.seed_packet_silhouette.clone()),
                        Node {
                            width: Val::Px(50.0),
                            height: Val::Px(70.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            ..default()
                        },
                    ));

                    // 有卡牌的槽位覆盖卡牌图 + 交互组件
                    if has_card {
                        let cost = kind.cost();
                        let image = card_image(kind, &assets);
                        parent.spawn((
                            ImageNode::new(image),
                            Node {
                                width: Val::Px(50.0),
                                height: Val::Px(70.0),
                                ..default()
                            },
                        ));
                        parent.spawn((
                            CardCooldownOverlay,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                width: Val::Px(50.0),
                                height: Val::Px(70.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                        ));
                        parent.spawn((
                            CardSelectedOverlay,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                width: Val::Px(50.0),
                                height: Val::Px(70.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        ));
                        parent.spawn((
                            Text::new(format!("{cost}")),
                            TextFont {
                                font: FontSource::Handle(font.clone()),
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.0, 0.0, 0.0)),
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: Val::Px(2.0),
                                right: Val::Px(18.0),
                                ..default()
                            },
                        ));
                    }
                });
            }
        });
}

fn update_sun_counter(bank: Res<SunBank>, mut query: Query<&mut Text, With<SunCounter>>) {
    if !bank.is_changed() {
        return;
    }
    for mut text in query.iter_mut() {
        **text = format!("{}", bank.amount);
    }
}

fn handle_card_click(
    _time: Res<Time>,
    mut selected: ResMut<crate::input::SelectedPlant>,
    bank: Res<SunBank>,
    cards: Res<PlantCards>,
    assets: Res<GameAssets>,
    mut commands: Commands,
    interaction_query: Query<(&Interaction, Entity, &PlantCard), Changed<Interaction>>,
) {
    for (interaction, _entity, card_data) in interaction_query.iter() {
        if let Interaction::Pressed = *interaction {
            let usable = bank.amount >= card_data.kind.cost() && cards.ready(&card_data.kind);
            if usable {
                selected.kind = Some(card_data.kind);
            } else {
                commands.spawn((
                    AudioPlayer::<AudioSource>(assets.cannot_choose_sound.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }
    }
}

fn sync_selected_overlay(
    selected: Res<crate::input::SelectedPlant>,
    cards: Query<(&PlantCard, &Children)>,
    mut overlay_query: Query<&mut BackgroundColor, With<CardSelectedOverlay>>,
) {
    for (card, children) in cards.iter() {
        let is_selected = selected.kind == Some(card.kind);
        for child in children.iter() {
            if let Ok(mut bg) = overlay_query.get_mut(child) {
                let alpha = if is_selected { 0.35 } else { 0.0 };
                bg.0 = Color::srgba(0.0, 0.0, 0.0, alpha);
            }
        }
    }
}

fn cooldown_tick(
    time: Res<Time>,
    mut cards: ResMut<PlantCards>,
    card_query: Query<(&PlantCard, &Children)>,
    mut overlay_query: Query<&mut BackgroundColor, With<CardCooldownOverlay>>,
    mut overlay_node_query: Query<&mut Node, With<CardCooldownOverlay>>,
) {
    cards.tick(time.delta_secs());

    for (card, children) in card_query.iter() {
        let remaining = cards.remaining(card.kind);
        let progress = if cards.ready(&card.kind) {
            0.0
        } else {
            (remaining / card.cooldown_duration).clamp(0.0, 1.0)
        };
        let overlay_height = progress * 70.0;
        for child in children.iter() {
            if let Ok(mut bg) = overlay_query.get_mut(child) {
                bg.0.set_alpha(0.6);
            }
            if let Ok(mut node) = overlay_node_query.get_mut(child) {
                node.height = Val::Px(overlay_height);
            }
        }
    }
}
