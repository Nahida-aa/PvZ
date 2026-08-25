use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::config::PauseMenuConfig;

use super::components::*;

pub(crate) fn spawn(
    panel: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    assets: &GameAssets,
    config: &PauseMenuConfig,
) {
    let cb = &config.continue_button;
    panel
        .spawn((
            Button,
            PauseButtonMarker,
            ContinueButton,
            OriginalTop(cb.top),
            ImageNode::new(assets.pause_return_button.clone()),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(cb.left),
                top: Val::Px(cb.top),
                width: Val::Px(cb.width),
                height: Val::Px(cb.height),
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
                    font_size: FontSize::Px(cb.font_size),
                    ..default()
                },
                TextColor(Color::srgb(
                    cb.text_color[0],
                    cb.text_color[1],
                    cb.text_color[2],
                )),
            ));
        });
}
