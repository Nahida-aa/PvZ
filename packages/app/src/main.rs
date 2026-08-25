use std::path::PathBuf;

use bevy::prelude::*;
use bevy::log::LogPlugin;
use bevy::window::WindowResolution;

use pvz_core::assets::BgmMusic;
use pvz_core::settings::AppConfig;
use pvz_core::level::LevelDefinition;
use pvz_core::ui::menu::pause_menu::config::PauseMenuConfig;

/// 默认关卡文件名（相对于 assets 目录）。
const DEFAULT_LEVEL: &str = "levels/test_day.ron";
/// 应用配置文件名（相对于 assets 目录）。
const APP_CONFIG: &str = "app.ron";
/// 暂停菜单布局文件名（相对于 assets 目录）。
const PAUSE_MENU_CONFIG: &str = "ui/pause_menu.ron";

/// 获取 assets 目录的绝对路径。
///
/// 基于可执行文件位置推算：binary 位于 `target/{profile}/pvz-app`，
/// 往上两级回到 workspace 根目录，再进入 `assets/`。
/// 这样无论从哪个目录运行都能正确找到资源。
fn assets_dir() -> PathBuf {
    let exe = std::env::current_exe().expect("无法获取可执行文件路径");
    exe.parent().unwrap().join("../../assets")
}

fn main() {
    let assets = assets_dir();
    let level_path = parse_level_arg(std::env::args().skip(1)).unwrap_or_else(|msg| {
        eprintln!("{msg}");
        std::process::exit(2);
    });

    let app_config_path = assets.join(APP_CONFIG);
    let level_path = assets.join(&level_path);
    let pause_menu_config_path = assets.join(PAUSE_MENU_CONFIG);
    let app_config = AppConfig::load_from_file(app_config_path.to_str().unwrap())
        .unwrap_or_else(|e| panic!("加载 {} 失败: {e}", app_config_path.display()));
    let level = LevelDefinition::load_from_file(level_path.to_str().unwrap())
        .unwrap_or_else(|e| panic!("加载 {} 失败: {e}", level_path.display()));
    let pause_menu_config =
        PauseMenuConfig::load_from_file(pause_menu_config_path.to_str().unwrap())
            .unwrap_or_else(|e| {
                eprintln!("加载 {} 失败: {e}，使用默认值", pause_menu_config_path.display());
                PauseMenuConfig::default()
            });

    let window_size = (app_config.win_w() as u32, app_config.win_h() as u32);

    App::new()
        .insert_resource(app_config)
        .insert_resource(level)
        .insert_resource(pause_menu_config)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Plants vs. Zombies".into(),
                resolution: WindowResolution::new(window_size.0, window_size.1),
                resizable: true,
                ..default()
            }),
            ..default()
        }).set(LogPlugin {
            filter: "info,icu_segmenter=error".into(),
            ..default()
        })        // Bevy AssetPlugin: 指定资源目录相对于 CWD 的路径。
        // 路径 "../../assets" 与 assets_dir() 保持一致，指向 workspace 根目录的 assets/。
        .set(AssetPlugin {
            file_path: "../../assets".into(),
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(pvz_core::CorePlugin)
        .add_systems(Startup, (setup_camera, start_music))
        .run();
}

/// 解析 `--level <path>` 参数，缺省返回默认关卡路径。
#[allow(clippy::never_loop)]
fn parse_level_arg(args: impl IntoIterator<Item = String>) -> Result<String, String> {
    let mut args = args.into_iter();
    for arg in args.by_ref() {
        match arg.as_str() {
            "--level" | "-l" => {
                return args
                    .next()
                    .ok_or_else(|| "--level 需要关卡 RON 路径".to_string());
            }
            other if other.starts_with("--level=") => {
                let path = other
                    .strip_prefix("--level=")
                    .filter(|p| !p.is_empty())
                    .ok_or_else(|| "--level= 需要关卡 RON 路径".to_string())?;
                return Ok(path.to_string());
            }
            unknown => return Err(format!("未知参数: {unknown}")),
        }
    }
    Ok(DEFAULT_LEVEL.to_string())
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::default(), GlobalTransform::default()));
}

/// 启动背景音乐，循环播放。
///
/// 音乐实体附加 `BgmMusic` 标记组件，供 `pause_menu.rs` 中的
/// `pause_bgm` / `resume_bgm` 系统查询 `AudioSink` 进行暂停/恢复控制。
fn start_music(mut commands: Commands, server: Res<AssetServer>, config: Res<AppConfig>) {
    commands.spawn((
        BgmMusic,
        AudioPlayer::<AudioSource>(server.load("music/dayLevel.ogg")),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(config.bgm_volume)),
    ));
}
