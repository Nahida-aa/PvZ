use bevy::prelude::*;
use crate::assets::GameAssets;
use crate::state::GameState;
use crate::ui::card_visual;

const ENCYCLOPEDIA_PLANTS: &[(&str, &str)] = &[
    ("Peashooter", "豌豆射手\n伤害: 20\n射速: 1.4s\n花费: 100"),
    ("Sunflower", "向日葵\n生产阳光\n花费: 50"),
    ("CherryBomb", "樱桃炸弹\n范围爆炸\n花费: 150"),
    ("WallNut", "坚果墙\n高耐久防御\n花费: 50"),
    ("PotatoMine", "土豆雷\n踩中爆炸\n花费: 25"),
    ("SnowPea", "寒冰射手\n减速+伤害\n花费: 175"),
    ("Chomper", "大嘴花\n一口吞食\n花费: 150"),
    ("RepeaterPea", "双发射手\n双倍火力\n花费: 200"),
    ("PuffShroom", "小喷菇\n免费短距\n花费: 0"),
    ("SunShroom", "阳光菇\n小→大阳光\n花费: 25"),
    ("FumeShroom", "大喷菇\n穿透攻击\n花费: 75"),
    ("GraveBuster", "咬咬墓碑\n清除墓碑\n花费: 75"),
    ("HypnoShroom", "催眠蘑菇\n催眠僵尸\n花费: 75"),
    ("IceShroom", "冰冻蘑菇\n全屏冻结\n花费: 75"),
    ("DoomShroom", "毁灭蘑菇\n大范围爆炸\n花费: 125"),
    ("LilyPad", "睡莲叶\n水面种植\n花费: 25"),
    ("Squash", "窝瓜\n碾压僵尸\n花费: 50"),
    ("ThreePeaShooter", "三线射手\n三路攻击\n花费: 325"),
    ("TangleKlep", "缠绕水草\n水中拉拽\n花费: 25"),
    ("Jalapeno", "火爆辣椒\n整行焚烧\n花费: 125"),
    ("Spikeweed", "地刺\n刺伤踩踏\n花费: 100"),
    ("TorchWood", "火炬树桩\n豌豆变火球\n花费: 175"),
    ("TallNut", "高坚果\n超高耐久\n花费: 125"),
    ("Seashroom", "水蘑菇\n水面免费\n花费: 0"),
    ("Plantern", "路灯花\n照亮迷雾\n花费: 25"),
    ("Cactus", "仙人掌\n穿刺攻击\n花费: 125"),
    ("Blover", "三叶草\n吹走迷雾\n花费: 100"),
    ("StarFruit", "杨桃\n五向攻击\n花费: 125"),
    ("PumpkinHead", "南瓜头\n额外护甲\n花费: 125"),
    ("Garlic", "大蒜\n改道僵尸\n花费: 50"),
    ("GiantWallNut", "巨型坚果\n超强防御\n花费: 125"),
];

#[derive(Component)]
pub struct EncyclopediaRoot;

#[derive(Component)]
pub struct EncyclopediaClose;

#[derive(Component)]
pub struct EncyclopediaPlantCard {
    pub name: String,
}

#[derive(Component)]
pub struct EncyclopediaDetailName;

#[derive(Component)]
pub struct EncyclopediaDetailDesc;

#[derive(Component)]
pub struct EncyclopediaDetailImage;

#[derive(Component)]
pub struct EncyclopediaDetailCost;

#[derive(Component)]
pub struct EncyclopediaCloseImage;

pub fn build_encyclopedia(commands: &mut Commands, assets: &GameAssets) {
    let panel_w = 800.0;
    let panel_h = 520.0;
    let ox = (1066.0 - panel_w) / 2.0;
    let oy = (600.0 - panel_h) / 2.0;

    commands
        .spawn((
            EncyclopediaRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(ox),
                top: Val::Px(oy),
                width: Val::Px(panel_w),
                height: Val::Px(panel_h),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ImageNode::new(assets.almanac_plant_back.clone()),
        ))
        .with_children(|root| {
            // ===== Left panel: plant grid (500px) =====
            root.spawn((
                Node {
                    width: Val::Px(500.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|left| {
                // Title
                left.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Text::new("植物图鉴"),
                        TextFont {
                            font_size: FontSize::Px(22.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

                // Scrollable grid
                left.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        padding: UiRect::all(Val::Px(8.0)),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                ))
                .with_children(|scroll_area| {
                    scroll_area
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            ..default()
                        },))
                        .with_children(|list| {
                            for chunk in ENCYCLOPEDIA_PLANTS.chunks(5) {
                                list.spawn((Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(4.0),
                                    row_gap: Val::Px(4.0),
                                    flex_wrap: FlexWrap::Wrap,
                                    ..default()
                                },))
                                .with_children(|row| {
                                    for &(name, _desc) in chunk {
                                        let card_img = card_visual::card_image_handle(name, assets);
                                        let cost = card_visual::plant_sun_cost(name);
                                        row.spawn((
                                            EncyclopediaPlantCard { name: name.to_string() },
                                            Button,
                                            Node {
                                                width: Val::Px(85.0),
                                                height: Val::Px(100.0),
                                                flex_direction: FlexDirection::Column,
                                                align_items: AlignItems::Center,
                                                padding: UiRect::all(Val::Px(4.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgba(0.12, 0.2, 0.12, 0.8)),
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
                                                    font_size: FontSize::Px(10.0),
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(1.0, 1.0, 0.5)),
                                            ));
                                        });
                                    }
                                });
                            }
                        });
                });
            });

            // ===== Right panel: detail area (300px) =====
            root.spawn((
                Node {
                    width: Val::Px(300.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.12, 0.08, 0.9)),
            ))
            .with_children(|right| {
                // Close button (top-right) — use Almanac_CloseButton.png
                right.spawn((
                    EncyclopediaClose,
                    Button,
                    Node {
                        width: Val::Px(89.0),
                        height: Val::Px(26.0),
                        align_self: AlignSelf::FlexEnd,
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                    ImageNode::new(assets.almanac_close_button.clone()),
                )).with_children(|p| {
                    p.spawn((
                        EncyclopediaCloseImage,
                        Node {
                            width: Val::Px(89.0),
                            height: Val::Px(26.0),
                            ..default()
                        },
                        ImageNode::new(assets.almanac_close_button.clone()),
                    ));
                });

                // Plant name
                right.spawn((
                    EncyclopediaDetailName,
                    Text::new("点击左侧植物查看"),
                    TextFont {
                        font_size: FontSize::Px(20.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ));

                // Plant image (large)
                right.spawn((
                    EncyclopediaDetailImage,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(150.0),
                        margin: UiRect::bottom(Val::Px(12.0)),
                        ..default()
                    },
                    ImageNode::new(assets.almanac_plant_card.clone()),
                ));

                // Description
                right.spawn((
                    EncyclopediaDetailDesc,
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(14.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.85, 0.85, 0.75)),
                ));

                // Cost
                right.spawn((
                    EncyclopediaDetailCost,
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(16.0),
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 1.0, 0.3)),
                    Node {
                        margin: UiRect::top(Val::Px(12.0)),
                        ..default()
                    },
                ));
            });
        });
}

pub fn handle_encyclopedia_close(
    interaction: Query<&Interaction, (Changed<Interaction>, With<EncyclopediaClose>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in interaction.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::ChoosingCards);
        }
    }
}

pub fn handle_encyclopedia_plant_click(
    interaction_query: Query<(&Interaction, &EncyclopediaPlantCard), Changed<Interaction>>,
    mut name_query: Query<&mut Text, With<EncyclopediaDetailName>>,
    mut desc_query: Query<&mut Text, With<EncyclopediaDetailDesc>>,
    mut cost_query: Query<&mut Text, With<EncyclopediaDetailCost>>,
    mut img_query: Query<&mut ImageNode, With<EncyclopediaDetailImage>>,
    assets: Res<GameAssets>,
) {
    for (interaction, card) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let (_, desc) = ENCYCLOPEDIA_PLANTS
                .iter()
                .find(|(n, _)| *n == card.name)
                .unwrap_or(&("", ""));
            let cost = card_visual::plant_sun_cost(&card.name);
            let card_img = card_visual::card_image_handle(&card.name, &assets);

            for mut text in name_query.iter_mut() {
                **text = card.name.clone();
            }
            for mut text in desc_query.iter_mut() {
                **text = desc.to_string();
            }
            for mut text in cost_query.iter_mut() {
                **text = format!("阳光消耗: {cost}");
            }
            for mut img in img_query.iter_mut() {
                img.image = card_img.clone();
            }

            info!("图鉴选中: {}", card.name);
        }
    }
}

pub fn handle_encyclopedia_close_hover(
    interaction: Query<(&Interaction, &Children), (Changed<Interaction>, With<EncyclopediaClose>)>,
    mut img_query: Query<&mut ImageNode, With<EncyclopediaCloseImage>>,
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
