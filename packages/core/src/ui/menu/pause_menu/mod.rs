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
pub mod config;
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
use bevy::ui::widget::TextShadow;

use crate::assets::GameAssets;
use crate::debug::DebugBorder;
use crate::settings::AppConfig;
use crate::state::GameState;

use self::config::PauseMenuConfig;
use self::config::png_dimensions;

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
    config: Res<PauseMenuConfig>,
    server: Res<AssetServer>,
) {
    bevy::log::info!(
        "setup_pause_menu: options_button left={}",
        config.options_button.left
    );
    build_pause_menu_ui(&mut commands, &assets, &app_config, &config, &server);
}

/// 构建暂停菜单 UI 的核心逻辑，供 setup 和热重载共用。
pub(crate) fn build_pause_menu_ui(
    commands: &mut Commands,
    assets: &GameAssets,
    app_config: &AppConfig,
    config: &PauseMenuConfig,
    server: &AssetServer,
) {
    let font = assets.font.clone();
    let oc = &config.overlay;
    let pc = &config.panel;
    let bg_handle: Handle<Image> = server.load(&pc.background_image);

    // 从 PNG 文件读取图片真实尺寸，作为 fallback
    let (img_w, img_h) = if let Ok(exe) = std::env::current_exe() {
        let img_path = exe.parent().unwrap()
            .join("../../assets").join(&pc.background_image);
        png_dimensions(img_path.to_str().unwrap())
            .unwrap_or((412, 483))
    } else {
        (412, 483)
    };
    // panel.width/height 覆盖 PNG 尺寸，None 时使用 PNG 原始尺寸
    let panel_w = pc.width.unwrap_or(img_w as f32);
    let panel_h = pc.height.unwrap_or(img_h as f32);

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
                        width: Val::Px(panel_w),
                        height: Val::Px(panel_h),
                        ..default()
                    },
                    ImageNode::new(bg_handle),
                    DebugBorder,
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
                        TextShadow {
                            offset: Vec2::new(-2.0, -2.0),
                            color: Color::BLACK,
                        },
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
