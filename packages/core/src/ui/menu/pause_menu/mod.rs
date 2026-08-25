//! 暂停菜单。
//!
//! # 暂停菜单布局 (option_dialog.png, 412×483)
//!
//! 对齐 Godot MainGame00Base.tscn 中的暂停菜单。
//! 所有布局参数从 `assets/ui/pause_menu.ron` 读取，支持热重载。
//!
//! # 状态流转
//!
//! - `Playing` ←Escape→ `Paused`（暂停/恢复切换）
//! - `Paused` → 点击"返回游戏" → `Playing`
//! - `Paused` → 点击"重新开始"/"主菜单" → 重置所有游戏状态 → `Playing`

mod almanac_button;
pub(crate) mod config;
mod components;
mod continue_button;
mod main_menu_button;
mod options_button;
mod restart_button;
pub(crate) mod systems;
mod ui_builders;

pub(crate) use components::*;

use bevy::prelude::*;
use bevy::ui::ZIndex;

use crate::assets::GameAssets;
use crate::settings::AppConfig;
use crate::state::GameState;

use self::config::PauseMenuConfig;

/// 暂停菜单插件。
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            systems::toggle_pause
                .run_if(in_state(GameState::Playing).or_eager(in_state(GameState::Paused))),
        )
        .add_systems(OnEnter(GameState::Paused), (setup_pause_menu, systems::pause_bgm))
        .add_systems(OnEnter(GameState::Playing), systems::resume_bgm)
        .add_systems(
            Update,
            (systems::handle_buttons, systems::handle_slider_values, systems::update_knob_positions, systems::play_button_sounds)
                .run_if(in_state(GameState::Paused)),
        )
        .add_systems(
            Update,
            systems::button_press_feedback.run_if(in_state(GameState::Paused)),
        )
        .add_systems(
            Update,
            systems::hot_reload_pause_menu.run_if(in_state(GameState::Paused)),
        )
        .add_systems(OnExit(GameState::Paused), systems::despawn_pause_menu)
        .add_systems(Update, systems::apply_time_scale);
    }
}

/// 构建暂停菜单 UI（系统入口）。
pub(crate) fn setup_pause_menu(
    mut commands: Commands,
    assets: Res<GameAssets>,
    app_config: Res<AppConfig>,
    pause_configs: Res<Assets<PauseMenuConfig>>,
) {
    let config = pause_configs
        .get(&assets.pause_menu_config)
        .cloned()
        .unwrap_or_default();
    build_pause_menu_ui(&mut commands, &assets, &app_config, &config);
}

/// 构建暂停菜单 UI 的核心逻辑，供 setup 和热重载共用。
pub(crate) fn build_pause_menu_ui(
    commands: &mut Commands,
    assets: &GameAssets,
    app_config: &AppConfig,
    config: &PauseMenuConfig,
) {
    let font = assets.font.clone();
    let oc = &config.overlay;
    let pc = &config.panel;

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
            BackgroundColor(Color::srgba(oc.color[0], oc.color[1], oc.color[2], oc.color[3])),
            ZIndex(oc.z_index),
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Relative,
                        width: Val::Px(pc.width),
                        height: Val::Px(pc.height),
                        ..default()
                    },
                    ImageNode::new(assets.pause_menu_bg.clone()),
                ))
                .with_children(|panel| {
                    // 音乐滑动条
                    let ms = &config.music_slider;
                    ui_builders::spawn_slider(
                        panel, ms.left, ms.top, ms.width, ms.height,
                        "音乐", SliderType::Music,
                        0.0, 1.0, 0.01, app_config.bgm_volume,
                        &font, &assets.slider_slot, &assets.slider_knob, &config,
                    );

                    // 音效滑动条
                    let ss = &config.sound_slider;
                    ui_builders::spawn_slider(
                        panel, ss.left, ss.top, ss.width, ss.height,
                        "音效", SliderType::Sound,
                        0.0, 1.0, 0.01, app_config.sfx_volume,
                        &font, &assets.slider_slot, &assets.slider_knob, &config,
                    );

                    // 倍速滑动条
                    let sp = &config.speed_slider;
                    ui_builders::spawn_slider(
                        panel, sp.left, sp.top, sp.width, sp.height,
                        "倍速", SliderType::Speed,
                        0.5, 3.0, 0.5, app_config.time_scale,
                        &font, &assets.slider_slot, &assets.slider_knob, &config,
                    );

                    // 倍速文字标签
                    let sl = &config.speed_label;
                    panel.spawn((
                        SpeedLabel,
                        Text::new(format!("{:.1} 倍", app_config.time_scale)),
                        TextFont {
                            font: FontSource::Handle(font.clone()),
                            font_size: FontSize::Px(sl.font_size),
                            ..default()
                        },
                        TextColor(Color::srgb(sl.color[0], sl.color[1], sl.color[2])),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(sl.left),
                            top: Val::Px(sl.top),
                            ..default()
                        },
                    ));

                    // 各按钮
                    almanac_button::spawn(panel, &font, &assets, &config);
                    options_button::spawn(panel, &font, &assets, &config);
                    restart_button::spawn(panel, &font, &assets, &config);
                    main_menu_button::spawn(panel, &font, &assets, &config);
                    continue_button::spawn(panel, &font, &assets, &config);
                });
        });
}
