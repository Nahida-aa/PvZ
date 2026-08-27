use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::ui::card_visual;

use super::components::*;
use super::data::AlmanacData;

pub fn build_encyclopedia(commands: &mut Commands, assets: &GameAssets, almanac: &AlmanacData) {
    let w = 1066.0_f32;
    let h = 600.0_f32;

    commands
        .spawn((
            EncyclopediaRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(w),
                height: Val::Px(h),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|root| {
            // Close button (always visible)
            root.spawn((
                AlmanacCloseButton,
                Button,
                GlobalZIndex(10),
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(89.0),
                    bottom: Val::Px(4.0),
                    width: Val::Px(89.0),
                    height: Val::Px(26.0),
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((
                    AlmanacCloseImage,
                    Node {
                        width: Val::Px(89.0),
                        height: Val::Px(26.0),
                        ..default()
                    },
                    ImageNode::new(assets.almanac_close_button.clone()),
                ));
            });

            build_index_page(root, assets, w, h);
            build_plant_page(root, assets, w, h, almanac);
        });
}

fn build_index_page(parent: &mut ChildSpawnerCommands, assets: &GameAssets, w: f32, h: f32) {
    parent
        .spawn((
            AlmanacIndexPage,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(w),
                height: Val::Px(h),
                ..default()
            },
            ImageNode {
                image: assets.almanac_index_back.clone(),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
        ))
        .with_children(|page| {
            page.spawn((
                Text::new("\u{56fe}\u{9274}\u{2014}\u{2014}\u{7d22}\u{5f15}"),
                TextFont {
                    font_size: FontSize::Px(33.0),
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(30.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));

            page.spawn((
                AlmanacPlantButton,
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(80.0),
                    top: Val::Px(180.0),
                    width: Val::Px(156.0),
                    height: Val::Px(42.0),
                    ..default()
                },
                ImageNode::new(assets.seed_chooser_button.clone()),
            ));

            page.spawn((
                AlmanacZombieButton,
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(80.0),
                    top: Val::Px(180.0),
                    width: Val::Px(156.0),
                    height: Val::Px(42.0),
                    ..default()
                },
                ImageNode::new(assets.small_button_bg.clone()),
            ));
        });
}

fn build_plant_page(
    parent: &mut ChildSpawnerCommands,
    assets: &GameAssets,
    w: f32,
    h: f32,
    almanac: &AlmanacData,
) {
    parent
        .spawn((
            AlmanacPlantPage,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(w),
                height: Val::Px(h),
                display: Display::None,
                ..default()
            },
            ImageNode {
                image: assets.almanac_plant_back.clone(),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
        ))
        .with_children(|page| {
            page.spawn((
                Text::new("\u{56fe}\u{9274}\u{2014}\u{2014}\u{690d}\u{7269}"),
                TextFont {
                    font_size: FontSize::Px(30.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.705, 0.0)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(30.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));

            // Card grid
            page.spawn((Node {
                position_type: PositionType::Absolute,
                left: Val::Px(27.0),
                top: Val::Px(89.0),
                width: Val::Px(414.0),
                height: Val::Px(470.0),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::auto(8),
                column_gap: Val::Px(2.0),
                row_gap: Val::Px(9.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },))
            .with_children(|grid| {
                for (card_name, _info) in &almanac.plants {
                    let card_img = card_visual::card_image_handle(card_name, assets);
                    let cost = card_visual::plant_sun_cost(card_name);
                    grid.spawn((
                        AlmanacPlantCard {
                            card_name: card_name.clone(),
                        },
                        Button,
                        Node {
                            width: Val::Px(50.0),
                            height: Val::Px(70.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Node {
                                width: Val::Px(50.0),
                                height: Val::Px(70.0),
                                ..default()
                            },
                            ImageNode::new(card_img),
                        ));
                        card.spawn((
                            Text::new(format!("{cost}")),
                            TextFont {
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.0, 0.0, 0.0)),
                            TextLayout::justify(Justify::Center),
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: Val::Px(2.0),
                                width: Val::Px(50.0),
                                ..default()
                            },
                        ));
                    });
                }
            });

            // Detail panel
            let panel_x = w - 317.0 - 156.0;
            let panel_y = (h - 476.0) / 2.0;
            page.spawn((Node {
                position_type: PositionType::Absolute,
                left: Val::Px(panel_x),
                top: Val::Px(panel_y),
                width: Val::Px(317.0),
                height: Val::Px(476.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },))
            .with_children(|panel| {
                // Character background
                panel.spawn((
                    AlmanacDetailBg,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(59.0),
                        top: Val::Px(8.0),
                        width: Val::Px(200.0),
                        height: Val::Px(200.0),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    ImageNode::new(assets.almanac_ground_day.clone()),
                ));

                // Preview image
                panel.spawn((
                    AlmanacDetailPreview,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(59.0),
                        top: Val::Px(8.0),
                        width: Val::Px(200.0),
                        height: Val::Px(200.0),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    ImageNode::new(Handle::default()),
                ));

                // PlantCard overlay
                panel.spawn((Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(324.0),
                    height: Val::Px(484.0),
                    ..default()
                },))
                .with_children(|p| {
                    p.spawn(ImageNode::new(assets.almanac_plant_card.clone()));
                });

                // Name
                spawn_text(panel, AlmanacNameText, "23.0", Color::srgb(0.772, 0.549, 0.152),
                    Justify::Center, 0.0, 179.0, 324.0, None);

                // Description
                spawn_text(panel, AlmanacDescText, "13.0", Color::srgb(0.156, 0.196, 0.352),
                    Justify::Left, 34.0, 215.0, 255.0, None);

                // Parameters
                spawn_text(panel, AlmanacParamsText, "13.0", Color::srgb(0.560, 0.262, 0.105),
                    Justify::Left, 34.0, 280.0, 255.0, None);

                // Hint
                panel.spawn((
                    AlmanacDetailText,
                    AlmanacHintText,
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.533, 0.196, 0.666)),
                    TextLayout {
                        justify: Justify::Left,
                        linebreak: LineBreak::WordBoundary,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(34.0),
                        top: Val::Px(340.0),
                        width: Val::Px(255.0),
                        ..default()
                    },
                ));

                // Introduction
                spawn_text(panel, AlmanacIntroText, "13.0", Color::srgb(0.560, 0.262, 0.105),
                    Justify::Left, 34.0, 370.0, 255.0, None);

                // Cost (bottom-left)
                panel.spawn((
                    AlmanacDetailText,
                    AlmanacCostText,
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.835, 0.141, 0.113)),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(26.0),
                        bottom: Val::Px(25.0),
                        ..default()
                    },
                ));

                // Cooltime (bottom-right)
                panel.spawn((
                    AlmanacDetailText,
                    AlmanacCooltimeText,
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.835, 0.141, 0.113)),
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(26.0),
                        bottom: Val::Px(25.0),
                        ..default()
                    },
                ));
            });

            // Return button
            page.spawn((
                AlmanacReturnButton,
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(16.0),
                    bottom: Val::Px(7.0),
                    width: Val::Px(164.0),
                    height: Val::Px(26.0),
                    ..default()
                },
                ImageNode::new(assets.almanac_index_button.clone()),
            ));
        });
}

fn spawn_text(
    parent: &mut ChildSpawnerCommands,
    marker: impl Component,
    font_size: &str,
    color: Color,
    justify: Justify,
    left: f32,
    top: f32,
    width: f32,
    height: Option<f32>,
) {
    let font_size_val: f32 = font_size.parse().unwrap_or(13.0);
    parent.spawn((
        AlmanacDetailText,
        marker,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(font_size_val),
            ..default()
        },
        TextColor(color),
        TextLayout {
            justify,
            linebreak: LineBreak::WordBoundary,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(left),
            top: Val::Px(top),
            width: Val::Px(width),
            height: height.map(Val::Px).unwrap_or_default(),
            ..default()
        },
    ));
}
