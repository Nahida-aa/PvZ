use bevy::audio::{AudioSink, PlaybackSettings};
use bevy::prelude::*;
use bevy::audio::AudioSource;
use bevy_ui_widgets::{Slider, SliderRange, SliderThumb, SliderValue};

use crate::assets::{BgmMusic, GameAssets};
use crate::settings::AppConfig;
use crate::input::SelectedPlant;
use crate::lawn::LawnOccupancy;
use crate::level::LevelRuntime;
use crate::state::{GameState, GameplayEntity};
use crate::ui::menebar::SunBank;
use crate::ui::plant_cards::PlantCards;
use super::config::PauseMenuConfig;

use super::components::*;
use super::super::despawn_recursive;

pub(crate) fn apply_time_scale(config: Res<AppConfig>, mut time: ResMut<Time<Virtual>>) {
    time.set_relative_speed(config.time_scale);
}

pub(crate) fn toggle_pause(
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

pub(crate) fn pause_bgm(mut sink: Query<&mut AudioSink, With<BgmMusic>>, config: Res<AppConfig>) {
    if let Ok(mut sink) = sink.single_mut() {
        sink.pause();
        sink.set_volume(bevy::audio::Volume::Linear(config.bgm_volume));
    }
}

pub(crate) fn resume_bgm(mut sink: Query<&mut AudioSink, With<BgmMusic>>, config: Res<AppConfig>) {
    if let Ok(mut sink) = sink.single_mut() {
        sink.set_volume(bevy::audio::Volume::Linear(config.bgm_volume));
        sink.play();
    }
}

/// 按钮按下时播放 gravebutton 音效。
pub(crate) fn play_button_sounds(
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

/// 根据 SliderValue 更新旋钮位置。
pub(crate) fn update_knob_positions(
    sliders: Query<(Entity, &SliderValue, &SliderRange, &Node), (With<Slider>, Without<SliderThumb>)>,
    children_query: Query<&Children>,
    mut knob_query: Query<(&mut Node, &SliderThumb), Without<Slider>>,
) {
    for (slider_entity, value, range, slider_node) in sliders.iter() {
        let ratio = range.thumb_position(value.0);
        let track_height: f32 = match slider_node.height {
            Val::Px(v) => v,
            _ => 10.0,
        };

        if let Ok(children) = children_query.get(slider_entity) {
            for child in children.iter() {
                if let Ok((mut knob_node, _)) = knob_query.get_mut(child) {
                    let knob_height: f32 = match knob_node.height {
                        Val::Px(v) => v,
                        _ => 29.0,
                    };
                    knob_node.left = Val::Percent(ratio * 100.0);
                    knob_node.top = Val::Px((track_height - knob_height) * 0.5);
                }
            }
        }
    }
}

/// 处理滑动条值变化，应用到 AppConfig。
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_slider_values(
    slider_types: Query<(&SliderType, &SliderValue), Changed<SliderValue>>,
    mut config: ResMut<AppConfig>,
    mut bgm_sink: Query<&mut AudioSink, With<BgmMusic>>,
    mut speed_label: Query<&mut Text, With<SpeedLabel>>,
) {
    for (slider_type, value) in slider_types.iter() {
        match slider_type {
            SliderType::Music => {
                config.bgm_volume = value.0;
                if let Ok(mut sink) = bgm_sink.single_mut() {
                    sink.set_volume(bevy::audio::Volume::Linear(value.0));
                }
            }
            SliderType::Sound => {
                config.sfx_volume = value.0;
            }
            SliderType::Speed => {
                config.time_scale = value.0;
                for mut text in speed_label.iter_mut() {
                    **text = format!("{:.1} 倍", value.0);
                }
            }
        }
    }
}

/// 处理暂停菜单按钮点击。
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_buttons(
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
    level: Res<crate::level::LevelDefinition>,
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

/// 按钮按下时的轻微位移反馈。
pub(crate) fn button_press_feedback(
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
pub(crate) fn despawn_pause_menu(
    mut commands: Commands,
    query: Query<Entity, With<PauseMenuRoot>>,
    children: Query<&Children>,
) {
    for entity in query.iter() {
        despawn_recursive(&mut commands, entity, &children);
    }
}

/// 热重载暂停菜单：检测 `pause_menu.ron` 文件变更，销毁旧 UI 并用新配置重建。
pub(crate) fn hot_reload_pause_menu(
    mut commands: Commands,
    assets: Res<GameAssets>,
    app_config: Res<AppConfig>,
    mut config: ResMut<PauseMenuConfig>,
    mut last_mtime: Local<Option<std::time::SystemTime>>,
    query: Query<Entity, With<PauseMenuRoot>>,
    children: Query<&Children>,
) {
    // 通过可执行文件位置推算 assets 路径（与 main.rs 的 assets_dir() 一致）
    let Ok(exe) = std::env::current_exe() else { return };
    let ron_path = exe
        .parent()
        .unwrap()
        .join("../../assets/ui/pause_menu.ron");

    let meta = std::fs::metadata(&ron_path).ok();
    let mtime = meta.and_then(|m| m.modified().ok());

    let changed = match (last_mtime.as_ref(), &mtime) {
        (Some(prev), Some(curr)) => *prev != *curr,
        (None, Some(curr)) => {
            // 首帧：初始化 mtime，不触发重建（setup_pause_menu 已处理）
            *last_mtime = Some(*curr);
            false
        }
        _ => false,
    };

    if let Some(t) = mtime {
        *last_mtime = Some(t);
    }

    if !changed {
        return;
    }

    bevy::log::info!("检测到 pause_menu.ron 变更，重建暂停菜单");

    // 重新读取 .ron 并更新 Resource
    if let Ok(new_config) = PauseMenuConfig::load_from_file(ron_path.to_str().unwrap()) {
        *config = new_config;
    }

    // 销毁旧 UI
    for entity in query.iter() {
        despawn_recursive(&mut commands, entity, &children);
    }

    bevy::log::info!(
        "热重载完成 — options_button left={}",
        config.options_button.left
    );
    super::build_pause_menu_ui(&mut commands, &assets, &app_config, &config);
}
