use bevy::prelude::*;
use bevy::window::WindowResolution;
use pvz_core::state::GameState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Plants vs. Zombies".into(),
                resolution: WindowResolution::new(800, 600), // .with_scale_factor_override(1.6)
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(pvz_core::CorePlugin)
        .add_systems(OnEnter(GameState::Playing), (setup_camera, start_music))
        .run();
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
