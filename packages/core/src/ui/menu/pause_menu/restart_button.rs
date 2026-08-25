use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::config::PauseMenuConfig;

use super::components::*;
use super::ui_builders;

pub(crate) fn spawn(
    panel: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    assets: &GameAssets,
    config: &PauseMenuConfig,
) {
    let b = &config.restart_button;
    ui_builders::spawn_small_button(
        panel,
        RestartButton,
        b.left, b.top, b.width, b.height,
        "重新开始",
        font,
        &assets.small_button_bg,
        config,
    );
}
