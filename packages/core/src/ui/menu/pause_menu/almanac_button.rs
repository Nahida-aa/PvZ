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
    let b = &config.almanac_button;
    ui_builders::spawn_small_button(
        panel,
        AlmanacButton,
        b.left, b.top, b.width, b.height,
        "查看图鉴",
        font,
        &assets.small_button_bg,
        config,
    );
}
