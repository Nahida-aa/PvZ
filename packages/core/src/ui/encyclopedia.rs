use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use crate::assets::GameAssets;
use crate::state::GameState;
use crate::ui::card_visual;

// ─── Data loading from per-plant files ───

#[derive(Deserialize)]
struct LocalInfo {
    name: String,
    description: String,
    intro: String,
    hint: Option<String>,
}

#[derive(Deserialize, Default)]
struct LocalData {
    sun_cost: Option<u32>,
    cooldown: Option<f64>,
    toughness: Option<u64>,
    damage: Option<u64>,
    interval: Option<f64>,
    bg: Option<String>,
    preview_image: Option<String>,
}

#[derive(Clone)]
pub struct PlantInfo {
    pub name: String,
    pub desc: String,
    pub intro: String,
    pub hint: Option<String>,
    pub params: HashMap<String, String>,
    pub bg: String,
    pub preview_image: String,
}

#[derive(Resource)]
pub struct AlmanacData {
    pub plants: Vec<(String, PlantInfo)>,
}

impl AlmanacData {
    pub fn load() -> Self {
        let assets_root = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("../../assets")))
            .unwrap_or_default();
        let plants_root = assets_root.join("plants");

        let mut entries: Vec<(String, PlantInfo)> = Vec::new();

        if let Ok(dirs) = std::fs::read_dir(&plants_root) {
            for dir in dirs.flatten() {
                if !dir.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    continue;
                }
                let dir_name = dir.file_name().to_string_lossy().to_string();
                let Some(card_name) = dir_to_card_name(&dir_name) else {
                    continue;
                };

                let path = dir.path();
                let Some(info) = load_local_info(&path) else {
                    continue;
                };
                let data = load_local_data(&path).unwrap_or_default();

                entries.push((card_name.to_string(), build_entry(card_name, info, data)));
            }
        }

        info!("图鉴加载了 {} 种植物", entries.len());
        Self { plants: entries }
    }
}

fn load_local_info(plant_dir: &Path) -> Option<LocalInfo> {
    let text = std::fs::read_to_string(plant_dir.join("info.zh.jsonc")).ok()?;
    jsonc_parse(&text).ok()
}

fn load_local_data(plant_dir: &Path) -> Option<LocalData> {
    let text = std::fs::read_to_string(plant_dir.join("data.jsonc")).ok()?;
    jsonc_parse(&text).ok()
}

fn jsonc_parse<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, String> {
    let val: serde_json::Value = jsonc_parser::parse_to_serde_value(text, &Default::default())
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "JSONC parse returned None".to_string())?;
    serde_json::from_value(val).map_err(|e| e.to_string())
}

fn dir_to_card_name(dir: &str) -> Option<&'static str> {
    match dir {
        "pea_shooter" => Some("Peashooter"),
        "sun_flower" => Some("Sunflower"),
        "cherry_bomb" => Some("CherryBomb"),
        "wall_nut" => Some("WallNut"),
        "potato_mine" => Some("PotatoMine"),
        "snow_pea" => Some("SnowPea"),
        "chomper" => Some("Chomper"),
        "repeater_pea" => Some("RepeaterPea"),
        "puff_shroom" => Some("PuffShroom"),
        "sun_shroom" => Some("SunShroom"),
        "fume_shroom" => Some("FumeShroom"),
        "grave_buster" => Some("GraveBuster"),
        "hypno_shroom" => Some("HypnoShroom"),
        "ice_shroom" => Some("IceShroom"),
        "doom_shroom" => Some("DoomShroom"),
        "lily_pad" => Some("LilyPad"),
        "squash" => Some("Squash"),
        "three_pea_shooter" => Some("ThreePeaShooter"),
        "tangle_klep" => Some("TangleKlep"),
        "jalapeno" => Some("Jalapeno"),
        "spike_weed" => Some("Spikeweed"),
        "torch_wood" => Some("TorchWood"),
        "tall_nut" => Some("TallNut"),
        "sea_shroom" => Some("Seashroom"),
        "plantern" => Some("Plantern"),
        "cactus" => Some("Cactus"),
        "blover" => Some("Blover"),
        "star_fruit" => Some("StarFruit"),
        "pumpkin" => Some("PumpkinHead"),
        "garlic" => Some("Garlic"),
        "giant_wall_nut" => Some("GiantWallNut"),
        _ => None,
    }
}

fn build_entry(card_name: &str, info: LocalInfo, data: LocalData) -> PlantInfo {
    let mut params: HashMap<String, String> = HashMap::new();
    if let Some(cost) = data.sun_cost {
        params.insert("\u{9633}\u{5149}\u{6d88}\u{8017}".into(), cost.to_string());
    }
    if let Some(cooldown) = data.cooldown {
        params.insert(
            "\u{51b7}\u{5374}\u{65f6}\u{95f4}".into(),
            format!("{cooldown}\u{79d2}"),
        );
    }
    if let Some(toughness) = data.toughness {
        params.insert("\u{751f}\u{547d}".into(), toughness.to_string());
    }
    if let Some(damage) = data.damage {
        params.insert("\u{4f24}\u{5bb3}".into(), damage.to_string());
    }
    if let Some(interval) = data.interval {
        params.insert(
            "\u{653b}\u{51fb}\u{95f4}\u{9694}".into(),
            format!("{interval}\u{79d2}"),
        );
    }

    PlantInfo {
        name: info.name,
        desc: info.description,
        intro: info.intro,
        hint: info.hint,
        params,
        bg: data.bg.unwrap_or_else(|| "Day".to_string()),
        preview_image: data
            .preview_image
            .unwrap_or_else(|| format!("plants/{}/parts/{}.png", card_name.to_lowercase(), card_name)),
    }
}

fn get_ground_bg(bg: &str, assets: &GameAssets) -> Handle<Image> {
    match bg {
        "Day" => assets.almanac_ground_day.clone(),
        "Night" => assets.almanac_ground_night.clone(),
        "Pool" => assets.almanac_ground_pool.clone(),
        "Fog" => assets.almanac_ground_fog.clone(),
        "Ice" => assets.almanac_ground_ice.clone(),
        "Roof" => assets.almanac_ground_roof.clone(),
        _ => assets.almanac_ground_day.clone(),
    }
}

// ─── Components ───

#[derive(Component)]
pub struct EncyclopediaRoot;

#[derive(Component)]
pub struct AlmanacIndexPage;

#[derive(Component)]
pub struct AlmanacPlantPage;

#[derive(Component)]
pub struct AlmanacPlantCard {
    pub card_name: String,
}

#[derive(Component)]
pub struct AlmanacPlantButton;

#[derive(Component)]
pub struct AlmanacZombieButton;

#[derive(Component)]
pub struct AlmanacReturnButton;

#[derive(Component)]
pub struct AlmanacCloseButton;

#[derive(Component)]
pub struct AlmanacCloseImage;

#[derive(Component)]
pub struct AlmanacDetailBg;

#[derive(Component)]
pub struct AlmanacDetailPreview;

#[derive(Component)]
pub struct AlmanacDetailName;

#[derive(Component)]
pub struct AlmanacDetailDesc;

#[derive(Component)]
pub struct AlmanacDetailParams;

#[derive(Component)]
pub struct AlmanacDetailHint;

#[derive(Component)]
pub struct AlmanacDetailIntro;

#[derive(Component)]
pub struct AlmanacDetailCost;

#[derive(Component)]
pub struct AlmanacDetailCooltime;

// ─── Build encyclopedia ───

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
            // === Close button (always visible) ===
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

            // === Index page ===
            build_index_page(root, assets, w, h);

            // === Plant page (hidden initially) ===
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
            ImageNode::new(assets.almanac_index_back.clone()),
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
            ImageNode::new(assets.almanac_plant_back.clone()),
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

            // Card grid (left side, 8 columns, 50x70 cards)
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

            // Detail panel (right side, 317x476)
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
                // Character background (200x200, clipped)
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

                // Preview image (on top of ground bg)
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

                // PlantCard overlay (324x484)
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

                // Plant name
                panel.spawn((
                    AlmanacDetailName,
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(23.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.772, 0.549, 0.152)),
                    TextLayout::justify(Justify::Center),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(179.0),
                        width: Val::Px(324.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                ));

                // Description
                panel.spawn((
                    AlmanacDetailDesc,
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.156, 0.196, 0.352)),
                    TextLayout {
                        justify: Justify::Left,
                        linebreak: LineBreak::WordBoundary,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(34.0),
                        top: Val::Px(215.0),
                        width: Val::Px(255.0),
                        ..default()
                    },
                ));

                // Parameters
                panel.spawn((
                    AlmanacDetailParams,
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.560, 0.262, 0.105)),
                    TextLayout {
                        justify: Justify::Left,
                        linebreak: LineBreak::WordBoundary,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(34.0),
                        top: Val::Px(280.0),
                        width: Val::Px(255.0),
                        ..default()
                    },
                ));

                // Hint (optional, purple)
                panel.spawn((
                    AlmanacDetailHint,
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
                        display: Display::None,
                        ..default()
                    },
                ));

                // Introduction
                panel.spawn((
                    AlmanacDetailIntro,
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.560, 0.262, 0.105)),
                    TextLayout {
                        justify: Justify::Left,
                        linebreak: LineBreak::WordBoundary,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(34.0),
                        top: Val::Px(370.0),
                        width: Val::Px(255.0),
                        ..default()
                    },
                ));

                // Cost (bottom-left)
                panel.spawn((
                    AlmanacDetailCost,
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
                    AlmanacDetailCooltime,
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

            // Return button (bottom-left)
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

// ─── Systems ───

pub fn handle_encyclopedia_close(
    interaction: Query<&Interaction, (Changed<Interaction>, With<AlmanacCloseButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in interaction.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::ChoosingCards);
        }
    }
}

pub fn handle_encyclopedia_close_hover(
    interaction: Query<(&Interaction, &Children), (Changed<Interaction>, With<AlmanacCloseButton>)>,
    mut img_query: Query<&mut ImageNode, With<AlmanacCloseImage>>,
    assets: Res<GameAssets>,
) {
    for (interaction, children) in interaction.iter() {
        for child in children.iter() {
            if let Ok(mut img) = img_query.get_mut(child) {
                match *interaction {
                    Interaction::Hovered => {
                        img.image = assets.almanac_close_button_highlight.clone();
                    }
                    _ => {
                        img.image = assets.almanac_close_button.clone();
                    }
                }
            }
        }
    }
}

pub fn handle_plant_button(
    interaction: Query<&Interaction, (Changed<Interaction>, With<AlmanacPlantButton>)>,
    mut index_page: Query<&mut Visibility, With<AlmanacIndexPage>>,
    mut plant_page: Query<&mut Visibility, With<AlmanacPlantPage>>,
) {
    for interaction in interaction.iter() {
        if *interaction == Interaction::Pressed {
            for mut vis in index_page.iter_mut() {
                *vis = Visibility::Hidden;
            }
            for mut vis in plant_page.iter_mut() {
                *vis = Visibility::Inherited;
            }
        }
    }
}

pub fn handle_return_button(
    interaction: Query<&Interaction, (Changed<Interaction>, With<AlmanacReturnButton>)>,
    mut index_page: Query<&mut Visibility, With<AlmanacIndexPage>>,
    mut plant_page: Query<&mut Visibility, With<AlmanacPlantPage>>,
) {
    for interaction in interaction.iter() {
        if *interaction == Interaction::Pressed {
            for mut vis in index_page.iter_mut() {
                *vis = Visibility::Inherited;
            }
            for mut vis in plant_page.iter_mut() {
                *vis = Visibility::Hidden;
            }
        }
    }
}

pub fn handle_plant_card_click(
    interaction_query: Query<(&Interaction, &AlmanacPlantCard), Changed<Interaction>>,
    almanac: Res<AlmanacData>,
    asset_server: Res<AssetServer>,
    mut bg_query: Query<&mut ImageNode, With<AlmanacDetailBg>>,
    mut preview_query: Query<&mut ImageNode, (With<AlmanacDetailPreview>, Without<AlmanacDetailBg>)>,
    mut name_query: Query<&mut Text, With<AlmanacDetailName>>,
    mut desc_query: Query<&mut Text, With<AlmanacDetailDesc>>,
    mut params_query: Query<&mut Text, With<AlmanacDetailParams>>,
    mut hint_query: Query<(&mut Text, &mut Node), With<AlmanacDetailHint>>,
    mut intro_query: Query<&mut Text, With<AlmanacDetailIntro>>,
    mut cost_query: Query<&mut Text, With<AlmanacDetailCost>>,
    mut cooltime_query: Query<&mut Text, With<AlmanacDetailCooltime>>,
    assets: Res<GameAssets>,
) {
    for (interaction, card) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some((_name, data)) = almanac.plants.iter().find(|(n, _)| n == &card.card_name) else {
            continue;
        };

        for mut img in bg_query.iter_mut() {
            img.image = get_ground_bg(&data.bg, &assets);
        }
        for mut img in preview_query.iter_mut() {
            img.image = asset_server.load(&data.preview_image);
        }
        for mut text in name_query.iter_mut() {
            **text = data.name.clone();
        }
        for mut text in desc_query.iter_mut() {
            **text = data.desc.clone();
        }
        for mut text in params_query.iter_mut() {
            let mut s = String::new();
            for (k, v) in &data.params {
                s.push_str(&format!("{k}: {v}\n"));
            }
            **text = s;
        }
        for (mut text, mut node) in hint_query.iter_mut() {
            if let Some(ref hint) = data.hint {
                **text = hint.clone();
                node.display = Display::Flex;
            } else {
                node.display = Display::None;
            }
        }
        for mut text in intro_query.iter_mut() {
            **text = data.intro.clone();
        }
        let cost = card_visual::plant_sun_cost(&card.card_name);
        for mut text in cost_query.iter_mut() {
            **text = format!("\u{82b1}\u{8d39}: {cost}");
        }
        for mut text in cooltime_query.iter_mut() {
            **text = "\u{51b7}\u{5374}\u{65f6}\u{95f4}: (\u{79d2})".to_string();
        }

        info!("\u{56fe}\u{9274}\u{9009}\u{4e2d}: {}", data.name);
    }
}
