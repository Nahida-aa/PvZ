use bevy::prelude::*;
use bevy::log::LogPlugin;
use bevy::window::WindowResolution;

use pvz_core::config::{AppConfig, LevelDefinition};

const DEFAULT_LEVEL_PATH: &str = "assets/levels/level_01.ron";
const APP_CONFIG_PATH: &str = "assets/app.ron";

fn main() {
    let level_path = parse_level_arg(std::env::args().skip(1)).unwrap_or_else(|msg| {
        eprintln!("{msg}");
        std::process::exit(2);
    });

    let app_config = AppConfig::load_from_file(APP_CONFIG_PATH)
        .unwrap_or_else(|e| panic!("加载 {APP_CONFIG_PATH} 失败: {e}"));
    let level = LevelDefinition::load_from_file(&level_path)
        .unwrap_or_else(|e| panic!("加载 {level_path} 失败: {e}"));

    let window_size = (app_config.win_w() as u32, app_config.win_h() as u32);

    App::new()
        .insert_resource(app_config)
        .insert_resource(level)
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
    Ok(DEFAULT_LEVEL_PATH.to_string())
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::default(), GlobalTransform::default()));
}

fn start_music(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::<AudioSource>(server.load("music/dayLevel.ogg")),
        PlaybackSettings::LOOP,
    ));
}
