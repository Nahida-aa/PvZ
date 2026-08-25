//! 暂停菜单 & 结算画面。
//!
//! # 暂停菜单布局 (option_dialog.png, 412×483)
//!
//! 对齐 Godot MainGame00Base.tscn 中的暂停菜单。
//!
//! # 状态流转
//!
//! - `Playing` ←Escape→ `Paused`（暂停/恢复切换）
//! - `Paused` → 点击"返回游戏" → `Playing`
//! - `Paused` → 点击"重新开始"/"主菜单" → 重置所有游戏状态 → `Playing`

use bevy::audio::{AudioSink, PlaybackSettings};
use bevy::prelude::*;
use bevy::ui::ZIndex;
use bevy::ui::prelude::{BorderRect, NodeImageMode, SliceScaleMode, TextureSlicer};
use bevy::audio::AudioSource;

use crate::assets::{BgmMusic, GameAssets};
use crate::config::AppConfig;
use crate::input::SelectedPlant;
use crate::lawn::LawnOccupancy;
use crate::level::LevelRuntime;
use crate::state::{GameState, GameplayEntity};
use crate::ui::menebar::SunBank;
use crate::ui::plant_cards::PlantCards;

// ===== 暂停菜单 =====

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component)]
struct PauseButtonMarker;

#[derive(Component)]
struct ContinueButton;

#[derive(Component)]
struct RestartButton;

#[derive(Component)]
struct MainMenuButton;

#[derive(Component)]
struct AlmanacButton;

#[derive(Component)]
struct OptionsButton;

/// 记录按钮未按下时的原始 `top`，用于按下时轻微位移反馈。
#[derive(Component)]
struct OriginalTop(pub f32);

/// 滑动条类型标记。
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum SliderType {
    Music,
    Sound,
    Speed,
}

/// 滑动条标记。
#[derive(Component)]
struct SliderMarker {
    slider_type: SliderType,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
}

/// 滑动条旋钮标记。
#[derive(Component)]
struct SliderKnob;

/// 倍速显示标签（"倍速 1.0 倍"）。
#[derive(Component)]
struct SpeedLabel;

/// 暂停菜单插件。
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            toggle_pause
                .run_if(in_state(GameState::Playing).or_eager(in_state(GameState::Paused))),
        )
        .add_systems(OnEnter(GameState::Paused), (setup_pause_menu, pause_bgm))
        .add_systems(OnEnter(GameState::Playing), resume_bgm)
        .add_systems(
            Update,
            (handle_buttons, handle_sliders, play_button_sounds)
                .run_if(in_state(GameState::Paused)),
        )
        .add_systems(
            Update,
            button_press_feedback.run_if(in_state(GameState::Paused)),
        )
        .add_systems(OnExit(GameState::Paused), despawn_pause_menu)
        .add_systems(Update, apply_time_scale);
    }
}

/// 将 AppConfig.time_scale 应用到 Bevy Time。
fn apply_time_scale(config: Res<AppConfig>, mut time: ResMut<Time<Virtual>>) {
    time.set_relative_speed(config.time_scale);
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    assets: Res<GameAssets>,
    mut commands: Commands,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    match state.get() {
        GameState::Playing => {
            commands.spawn((
                AudioPlayer::<AudioSource>(assets.pause_sound.clone()),
                PlaybackSettings::DESPAWN,
            ));
            next.set(GameState::Paused);
        }
        GameState::Paused => {
            commands.spawn((
                AudioPlayer::<AudioSource>(assets.pause_sound.clone()),
                PlaybackSettings::DESPAWN,
            ));
            next.set(GameState::Playing);
        }
        _ => {}
    }
}

fn pause_bgm(mut sink: Query<&mut AudioSink, With<BgmMusic>>, config: Res<AppConfig>) {
    if let Ok(mut sink) = sink.single_mut() {
        sink.pause();
        sink.set_volume(bevy::audio::Volume::Linear(config.bgm_volume));
    }
}

fn resume_bgm(mut sink: Query<&mut AudioSink, With<BgmMusic>>, config: Res<AppConfig>) {
    if let Ok(mut sink) = sink.single_mut() {
        sink.set_volume(bevy::audio::Volume::Linear(config.bgm_volume));
        sink.play();
    }
}

/// 按钮按下时播放 gravebutton 音效。
fn play_button_sounds(
    interaction: Query<&Interaction, (Changed<Interaction>, With<PauseButtonMarker>)>,
    assets: Res<GameAssets>,
    mut commands: Commands,
) {
    for interaction in interaction.iter() {
        if *interaction == Interaction::Pressed {
            commands.spawn((
                AudioPlayer::<AudioSource>(assets.gravebutton_sound.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

/// 创建暂停菜单中的小按钮（"重新开始"、"主菜单"、"查看图鉴"、"选项设置"）。
fn spawn_small_button(
    parent: &mut ChildSpawnerCommands,
    btn_marker: impl Bundle,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    label: &str,
    font: &Handle<Font>,
    bg: &Handle<Image>,
) {
    parent
        .spawn((
            Button,
            PauseButtonMarker,
            btn_marker,
            OriginalTop(top),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                width: Val::Px(width),
                height: Val::Px(height),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|b| {
            b.spawn((
                ImageNode {
                    image: bg.clone(),
                    image_mode: NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect {
                            min_inset: Vec2::new(16.0, 16.0),
                            max_inset: Vec2::new(16.0, 20.0),
                        },
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: 100.0,
                    }),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(-6.5),
                    width: Val::Px(195.0),
                    height: Val::Px(50.0),
                    ..default()
                },
            ));
            b.spawn((
                Text::new(label),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });
}

/// 创建滑动条（对齐 Godot HSlider + 自定义主题）。
///
/// 布局：一行内左边 Label + 右边滑动条（track + knob）。
fn spawn_slider(
    parent: &mut ChildSpawnerCommands,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    label: &str,
    slider_type: SliderType,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    font: &Handle<Font>,
    slot_image: &Handle<Image>,
    knob_image: &Handle<Image>,
) {
    let track_width = width - 62.0;
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                width: Val::Px(width),
                height: Val::Px(height),
                ..default()
            },
        ))
        .with_children(|row| {
            // Label（蓝灰色，对齐 Godot Label2）
            row.spawn((
                Text::new(label),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.38, 0.384, 0.502)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(8.0),
                    ..default()
                },
            ));

            // 滑动条轨道
            let ratio = (value - min) / (max - min);
            row.spawn((
                SliderMarker { slider_type, min, max, step, value },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(62.0),
                    top: Val::Px(10.0),
                    width: Val::Px(track_width),
                    height: Val::Px(10.0),
                    ..default()
                },
                ImageNode {
                    image: slot_image.clone(),
                    image_mode: NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect {
                            min_inset: Vec2::new(0.0, 5.0),
                            max_inset: Vec2::new(0.0, 5.0),
                        },
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: 100.0,
                    }),
                    ..default()
                },
            ))
            .with_children(|track| {
                // 旋钮
                track.spawn((
                    SliderKnob,
                    ImageNode::new(knob_image.clone()),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(ratio * (track_width - 22.0)),
                        top: Val::Px(-10.0),
                        width: Val::Px(22.0),
                        height: Val::Px(29.0),
                        ..default()
                    },
                ));
            });
        });
}

/// 构建暂停菜单 UI。
///
/// 布局对齐 Godot MainGame00Base.tscn 中的暂停菜单。
/// Option 容器左上角在面板 (97, 112.5)，尺寸 218×258。
fn setup_pause_menu(mut commands: Commands, assets: Res<GameAssets>, config: Res<AppConfig>) {
    let font = assets.font.clone();
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(1066.0),
                height: Val::Px(600.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            ZIndex(1000),
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Relative,
                        width: Val::Px(412.0),
                        height: Val::Px(483.0),
                        ..default()
                    },
                    ImageNode::new(assets.pause_menu_bg.clone()),
                ))
                .with_children(|panel| {
                    // ===== Option 容器 (97, 112.5) 218×258 =====

                    // 音乐滑动条
                    spawn_slider(
                        panel,
                        97.0, 112.5, 218.0, 30.0,
                        "音乐",
                        SliderType::Music,
                        0.0, 1.0, 0.01, config.bgm_volume,
                        &font,
                        &assets.slider_slot,
                        &assets.slider_knob,
                    );

                    // 音效滑动条
                    spawn_slider(
                        panel,
                        97.0, 152.5, 218.0, 30.0,
                        "音效",
                        SliderType::Sound,
                        0.0, 1.0, 0.01, config.sfx_volume,
                        &font,
                        &assets.slider_slot,
                        &assets.slider_knob,
                    );

                    // 倍速滑动条
                    spawn_slider(
                        panel,
                        97.0, 192.5, 218.0, 30.0,
                        "倍速",
                        SliderType::Speed,
                        0.5, 3.0, 0.5, config.time_scale,
                        &font,
                        &assets.slider_slot,
                        &assets.slider_knob,
                    );

                    // 倍速文字标签（显示 "1.0 倍"）
                    panel.spawn((
                        SpeedLabel,
                        Text::new(format!("{:.1} 倍", config.time_scale)),
                        TextFont {
                            font: FontSource::Handle(font.clone()),
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.38, 0.384, 0.502)),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(175.0),
                            top: Val::Px(192.5 + 8.0),
                            ..default()
                        },
                    ));

                    // "查看图鉴" 按钮 (6, 123.5) 88×37
                    spawn_small_button(
                        panel,
                        AlmanacButton,
                        103.0, 236.0, 88.0, 37.0,
                        "查看图鉴",
                        &font,
                        &assets.small_button_bg,
                    );

                    // "选项设置" 按钮 (113, 124) 88×37
                    spawn_small_button(
                        panel,
                        OptionsButton,
                        195.0, 236.0, 88.0, 37.0,
                        "选项设置",
                        &font,
                        &assets.small_button_bg,
                    );

                    // "重新开始" 按钮 (5, 167) 195×37
                    spawn_small_button(
                        panel,
                        RestartButton,
                        102.0, 279.5, 195.0, 37.0,
                        "重新开始",
                        &font,
                        &assets.small_button_bg,
                    );

                    // "主菜单" 按钮 (5, 211) 195×37
                    spawn_small_button(
                        panel,
                        MainMenuButton,
                        102.0, 323.5, 195.0, 37.0,
                        "主菜单",
                        &font,
                        &assets.small_button_bg,
                    );

                    // "返回游戏" 按钮：贴底居中，btn_dialog_back_2.png (360×100)
                    panel
                        .spawn((
                            Button,
                            PauseButtonMarker,
                            ContinueButton,
                            OriginalTop(383.0),
                            ImageNode::new(assets.pause_return_button.clone()),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(26.0),
                                top: Val::Px(383.0),
                                width: Val::Px(360.0),
                                height: Val::Px(100.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("返回游戏"),
                                TextFont {
                                    font: FontSource::Handle(font.clone()),
                                    font_size: FontSize::Px(50.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.0, 1.0, 0.0)),
                            ));
                        });
                });
        });
}

/// 处理暂停菜单按钮点击。
#[allow(clippy::too_many_arguments)]
fn handle_buttons(
    interaction: Query<(&Interaction, Entity), (Changed<Interaction>, With<Button>)>,
    continue_buttons: Query<Entity, With<ContinueButton>>,
    restart_buttons: Query<Entity, With<RestartButton>>,
    mainmenu_buttons: Query<Entity, With<MainMenuButton>>,
    gameplay: Query<Entity, With<GameplayEntity>>,
    children: Query<&Children>,
    mut selected: ResMut<SelectedPlant>,
    mut sun: ResMut<SunBank>,
    mut cards: ResMut<PlantCards>,
    mut runtime: ResMut<LevelRuntime>,
    mut occupancy: ResMut<LawnOccupancy>,
    level: Res<crate::config::LevelDefinition>,
    mut next: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    for (interaction, entity) in interaction.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if continue_buttons.get(entity).is_ok() {
            next.set(GameState::Playing);
        } else if restart_buttons.get(entity).is_ok()
            || mainmenu_buttons.get(entity).is_ok()
        {
            let entities: Vec<Entity> = gameplay.iter().collect();
            for e in entities {
                despawn_recursive(&mut commands, e, &children);
            }
            selected.kind = None;
            *sun = SunBank::default();
            *cards = PlantCards::default();
            *runtime = LevelRuntime::default();
            *occupancy = LawnOccupancy::from_level(&level);
            next.set(GameState::Playing);
        }
    }
}

/// 处理滑动条拖拽。
fn handle_sliders(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut sliders: Query<(Entity, &mut SliderMarker, &Node)>,
    children_query: Query<&Children>,
    mut knob_query: Query<&mut Node, With<SliderKnob>>,
    mut speed_label: Query<&mut Text, With<SpeedLabel>>,
    mut config: ResMut<AppConfig>,
    mut bgm_sink: Query<&mut AudioSink, With<BgmMusic>>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    for (entity, mut marker, node) in sliders.iter_mut() {
        let node_left = match node.left {
            Val::Px(v) => v,
            _ => continue,
        };
        let node_width = match node.width {
            Val::Px(v) => v,
            _ => continue,
        };

        let track_left = node_left + 62.0;
        let track_width = node_width - 62.0;

        if cursor.x < track_left || cursor.x > track_left + track_width {
            continue;
        }

        let ratio = ((cursor.x - track_left) / track_width).clamp(0.0, 1.0);
        let raw_value = marker.min + ratio * (marker.max - marker.min);
        let snapped = (raw_value / marker.step).round() * marker.step;
        let new_value = snapped.clamp(marker.min, marker.max);

        if (new_value - marker.value).abs() < f32::EPSILON {
            continue;
        }

        marker.value = new_value;

        // 更新旋钮位置
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                if let Ok(mut knob_node) = knob_query.get_mut(child) {
                    knob_node.left = Val::Px(ratio * (track_width - 22.0));
                }
            }
        }

        match marker.slider_type {
            SliderType::Music => {
                config.bgm_volume = new_value;
                if let Ok(mut sink) = bgm_sink.single_mut() {
                    sink.set_volume(bevy::audio::Volume::Linear(new_value));
                }
            }
            SliderType::Sound => {
                config.sfx_volume = new_value;
            }
            SliderType::Speed => {
                config.time_scale = new_value;
                for mut text in speed_label.iter_mut() {
                    **text = format!("{:.1} 倍", new_value);
                }
            }
        }
    }
}

/// 按钮按下时的轻微位移反馈。
fn button_press_feedback(
    mut buttons: Query<(&Interaction, &OriginalTop, &mut Node), With<PauseButtonMarker>>,
) {
    for (interaction, original, mut node) in buttons.iter_mut() {
        let offset = if *interaction == Interaction::Pressed {
            2.0
        } else {
            0.0
        };
        node.top = Val::Px(original.0 + offset);
    }
}

/// 销毁暂停菜单 UI。
fn despawn_pause_menu(
    mut commands: Commands,
    query: Query<Entity, With<PauseMenuRoot>>,
    children: Query<&Children>,
) {
    for entity in query.iter() {
        despawn_recursive(&mut commands, entity, &children);
    }
}

/// 递归销毁实体及其所有子实体。
fn despawn_recursive(
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
) {
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            despawn_recursive(commands, child, children_query);
        }
    }
    commands.entity(entity).despawn();
}

// ===== 结算画面 (Victory / Defeat) =====

#[derive(Component)]
struct EndScreenRoot;

#[derive(Component)]
struct EndScreenButton;

pub struct EndScreenPlugin;

impl Plugin for EndScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Defeat), setup_end_screen)
            .add_systems(OnEnter(GameState::Victory), setup_end_screen)
            .add_systems(
                Update,
                handle_end_screen
                    .run_if(in_state(GameState::Defeat).or_eager(in_state(GameState::Victory))),
            )
            .add_systems(OnExit(GameState::Defeat), despawn_end_screen)
            .add_systems(OnExit(GameState::Victory), despawn_end_screen);
    }
}

fn setup_end_screen(
    mut commands: Commands,
    state: Res<State<GameState>>,
    assets: Res<GameAssets>,
) {
    let (title, color) = match state.get() {
        GameState::Victory => ("胜利！", Color::srgb(0.95, 0.85, 0.3)),
        _ => ("游戏失败", Color::srgb(0.85, 0.2, 0.2)),
    };
    let font = assets.font.clone();
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(1066.0),
                height: Val::Px(600.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ZIndex(1000),
            EndScreenRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(64.0),
                    ..default()
                },
                TextColor(color),
            ));
            parent
                .spawn((
                    Button,
                    EndScreenButton,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(60.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.82, 0.78, 0.62)),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("重新开始"),
                        TextFont {
                            font: FontSource::Handle(font.clone()),
                            font_size: FontSize::Px(26.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.15, 0.12, 0.05)),
                    ));
                });
        });
}

#[allow(clippy::too_many_arguments)]
fn handle_end_screen(
    interaction: Query<(&Interaction, Entity), (Changed<Interaction>, With<EndScreenButton>)>,
    gameplay: Query<Entity, With<GameplayEntity>>,
    children: Query<&Children>,
    mut selected: ResMut<SelectedPlant>,
    mut sun: ResMut<SunBank>,
    mut cards: ResMut<PlantCards>,
    mut runtime: ResMut<LevelRuntime>,
    mut occupancy: ResMut<LawnOccupancy>,
    level: Res<crate::config::LevelDefinition>,
    mut next: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    for (inter, entity) in interaction.iter() {
        if *inter != Interaction::Pressed {
            continue;
        }
        let _ = entity;
        let entities: Vec<Entity> = gameplay.iter().collect();
        for e in entities {
            despawn_recursive(&mut commands, e, &children);
        }
        selected.kind = None;
        *sun = SunBank::default();
        *cards = PlantCards::default();
        *runtime = LevelRuntime::default();
        *occupancy = LawnOccupancy::from_level(&level);
        next.set(GameState::Playing);
    }
}

fn despawn_end_screen(
    mut commands: Commands,
    query: Query<Entity, With<EndScreenRoot>>,
    children: Query<&Children>,
) {
    for entity in query.iter() {
        despawn_recursive(&mut commands, entity, &children);
    }
}
