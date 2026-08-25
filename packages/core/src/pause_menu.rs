//! 暂停菜单 & 结算画面。
//!
//! # 暂停菜单布局 (option_dialog.png, 412×483)
//!
//! ```text
//! +--------------------------------------------+
//! |  (半透明黑色遮罩 1066×600, ZIndex=1000)     |
//! |                                            |
//! |    +-- panel (412×483, option_dialog.png) -+|
//! |    |                                      ||
//! |    |  [重新开始]  195×37, 居中             ||
//! |    |  [主菜单]    195×37, 居中             ||
//! |    |                                      ||
//! |    |  [==== 返回游戏 360×100 ====] 贴底    ||
//! |    +--------------------------------------+|
//! +--------------------------------------------+
//! ```
//!
//! # 状态流转
//!
//! - `Playing` ←Escape→ `Paused`（暂停/恢复切换）
//! - `Paused` → 点击"返回游戏" → `Playing`
//! - `Paused` → 点击"重新开始"/"主菜单" → 重置所有游戏状态 → `Playing`
//!
//! # 音频控制
//!
//! - 进入 `Paused` 时暂停 BGM（`pause_bgm`）
//! - 进入 `Playing` 时恢复 BGM（`resume_bgm`）
//! - 音乐实体通过 `BgmMusic` 标记组件查询

use bevy::audio::AudioSink;
use bevy::prelude::*;
use bevy::ui::ZIndex;

use crate::assets::{BgmMusic, GameAssets};
use crate::components::menebar::SunBank;
use crate::components::plant_cards::PlantCards;
use crate::input::SelectedPlant;
use crate::config::LevelDefinition;
use crate::lawn::LawnOccupancy;
use crate::level::LevelRuntime;
use crate::state::{GameState, GameplayEntity};

// ===== 暂停菜单 =====

/// 暂停菜单根节点标记，用于整体销毁。
#[derive(Component)]
struct PauseMenuRoot;

/// 暂停菜单按钮通用标记，用于区分游戏内其他按钮。
#[derive(Component)]
struct PauseButtonMarker;

/// "返回游戏" 按钮标记。
#[derive(Component)]
struct ContinueButton;

/// "重新开始" 按钮标记。
#[derive(Component)]
struct RestartButton;

/// "主菜单" 按钮标记。
#[derive(Component)]
struct MainMenuButton;

/// 暂停菜单插件。
///
/// 注册以下系统：
/// - `toggle_pause`：Escape 键切换暂停/恢复
/// - `setup_pause_menu`：进入 Paused 时创建 UI
/// - `pause_bgm` / `resume_bgm`：暂停/恢复背景音乐
/// - `handle_buttons`：处理按钮点击
/// - `despawn_pause_menu`：退出 Paused 时销毁 UI
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
            handle_buttons.run_if(in_state(GameState::Paused)),
        )
        .add_systems(OnExit(GameState::Paused), despawn_pause_menu);
    }
}

/// Escape 键切换暂停状态。
///
/// 仅在 `Playing` ↔ `Paused` 之间切换，其他状态忽略。
fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    match state.get() {
        GameState::Playing => next.set(GameState::Paused),
        GameState::Paused => next.set(GameState::Playing),
        _ => {}
    }
}

/// 暂停背景音乐。
///
/// 查询带 `BgmMusic` 标记的 `AudioSink` 组件，调用 `pause()` 暂停播放。
fn pause_bgm(sink: Query<&AudioSink, With<BgmMusic>>) {
    if let Ok(sink) = sink.single() {
        sink.pause();
    }
}

/// 恢复背景音乐。
///
/// 进入 `Playing` 状态时触发，包括：
/// - 从 Paused 恢复
/// - 重新开始/主菜单后重新进入 Playing
/// - 游戏首次启动（Startup 阶段音乐已创建，此时 sink 可能尚未就绪，`single()` 会失败但无影响）
fn resume_bgm(sink: Query<&AudioSink, With<BgmMusic>>) {
    if let Ok(sink) = sink.single() {
        sink.play();
    }
}

/// 创建暂停菜单中的小按钮（"重新开始"、"主菜单"）。
///
/// 按钮使用绝对定位，背景图片为 `bg`，文字居中显示。
///
/// # 参数
/// - `left`, `top`：按钮在面板内的绝对像素坐标
/// - `width`, `height`：按钮尺寸（像素）
/// - `label`：按钮文字
/// - `font`：字体句柄
/// - `bg`：背景图片句柄（button_BG.png）
fn spawn_small_button(
    parent: &mut ChildSpawnerCommands,
    marker: impl Bundle,
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
            marker,
            ImageNode::new(bg.clone()),
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
                Text::new(label),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::srgb(0.2, 0.15, 0.05)),
            ));
        });
}

/// 构建暂停菜单 UI。
///
/// 层级结构：
/// 1. `PauseMenuRoot`：全屏半透明遮罩（1066×600, ZIndex=1000）
/// 2. Panel：暂停菜单面板（412×483, option_dialog.png）
/// 3. 按钮：重新开始 / 主菜单 / 返回游戏
///
/// 所有坐标基于 Godot 参考项目 `MainGame00Base.tscn` 中的暂停菜单布局。
fn setup_pause_menu(mut commands: Commands, assets: Res<GameAssets>) {
    let font = assets.font.clone();
    commands
        .spawn((
            // 全屏遮罩层：绝对定位覆盖整个窗口，半透明黑色背景
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
            // 暂停菜单面板：option_dialog.png (412×483)
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
                    // "重新开始" 按钮：水平居中 (412-195)/2=108.5≈109
                    spawn_small_button(
                        panel,
                        RestartButton,
                        109.0,
                        160.0,
                        195.0,
                        37.0,
                        "重新开始",
                        &font,
                        &assets.small_button_bg,
                    );
                    // "主菜单" 按钮：紧挨"重新开始"下方，间距 10px
                    spawn_small_button(
                        panel,
                        MainMenuButton,
                        109.0,
                        207.0,
                        195.0,
                        37.0,
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
                            ImageNode::new(assets.pause_return_button.clone()),
                            Node {
                                position_type: PositionType::Absolute,
                                // (412-360)/2 = 26，水平居中
                                left: Val::Px(26.0),
                                // 483-100 = 383，贴底
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
                                    font_size: FontSize::Px(28.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.1, 0.1, 0.1)),
                            ));
                        });
                });
        });
}

/// 处理暂停菜单按钮点击。
///
/// - **返回游戏**：直接切换到 `Playing`
/// - **重新开始 / 主菜单**：销毁所有 `GameplayEntity`，重置游戏资源（阳光、卡牌、关卡状态、草坪占用），切换到 `Playing`
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
    level: Res<LevelDefinition>,
    mut next: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    for (interaction, entity) in interaction.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if continue_buttons.get(entity).is_ok() {
            // "返回游戏"：恢复游戏状态
            next.set(GameState::Playing);
        } else if restart_buttons.get(entity).is_ok()
            || mainmenu_buttons.get(entity).is_ok()
        {
            // "重新开始" / "主菜单"：销毁所有游戏实体，重置资源
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

/// 销毁暂停菜单 UI（退出 Paused 状态时触发）。
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

/// 结算画面根节点标记。
#[derive(Component)]
struct EndScreenRoot;

/// 结算画面"重新开始"按钮标记。
#[derive(Component)]
struct EndScreenButton;

/// 结算画面插件。
///
/// 在 `Victory` 或 `Defeat` 状态时显示结算界面，包含标题和"重新开始"按钮。
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

/// 构建结算画面 UI。
///
/// 布局：全屏半透明遮罩 → 居中标题 + "重新开始"按钮（Flexbox 纵向排列）。
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
            // 标题文字
            parent.spawn((
                Text::new(title),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(64.0),
                    ..default()
                },
                TextColor(color),
            ));
            // "重新开始" 按钮
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

/// 处理结算画面"重新开始"按钮点击。
///
/// 销毁所有 `GameplayEntity`，重置游戏资源，切换到 `Playing` 状态。
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
    level: Res<LevelDefinition>,
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

/// 销毁结算画面 UI（退出 Victory/Defeat 状态时触发）。
fn despawn_end_screen(
    mut commands: Commands,
    query: Query<Entity, With<EndScreenRoot>>,
    children: Query<&Children>,
) {
    for entity in query.iter() {
        despawn_recursive(&mut commands, entity, &children);
    }
}
