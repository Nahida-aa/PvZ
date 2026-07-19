use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use pvz_core::state::GameState;
use pvz_core::CorePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Plants vs. Zombies".into(),
                resolution: WindowResolution::new(800, 600),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CorePlugin)
        .add_systems(OnEnter(GameState::Playing), (setup_camera, start_music))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::default(),
        GlobalTransform::default(),
    ));
}

fn start_music(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::<AudioSource>(server.load("music/dayLevel.ogg")),
        PlaybackSettings::LOOP,
    ));
}
