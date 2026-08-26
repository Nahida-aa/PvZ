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
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ImageNode::new(assets.almanac_plant_back.clone()),
        ))
        .with_children(|root| {
            // Title bar
            root.spawn((
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
                    Text::new("图鉴 -- 植物"),
                    TextFont {
                        font_size: FontSize::Px(22.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // Close button (top-right)
            root.spawn((
                EncyclopediaClose,
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(10.0),
                    top: Val::Px(5.0),
                    width: Val::Px(80.0),
                    height: Val::Px(30.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.6, 0.2, 0.2, 0.8)),
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new("关闭"),
                    TextFont {
                        font_size: FontSize::Px(16.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // Plant grid
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    padding: UiRect::all(Val::Px(10.0)),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
            ))
            .with_children(|scroll_area| {
                scroll_area
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            ..default()
                        },
                    ))
                    .with_children(|list| {
                        for chunk in ENCYCLOPEDIA_PLANTS.chunks(4) {
                            list.spawn((Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(100.0),
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            },))
                            .with_children(|row| {
                                for &(name, desc) in chunk {
                                    row.spawn((
                                        Node {
                                            width: Val::Px(180.0),
                                            height: Val::Px(100.0),
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Center,
                                            padding: UiRect::all(Val::Px(6.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.15, 0.25, 0.15, 0.8)),
                                    ))
                                    .with_children(|card| {
                                        let card_img = card_visual::card_image_handle(name, assets);
                                        card.spawn((
                                            Node {
                                                width: Val::Px(50.0),
                                                height: Val::Px(70.0),
                                                ..default()
                                            },
                                            ImageNode::new(card_img),
                                        ));
                                        card.spawn((
                                            Text::new(desc),
                                            TextFont {
                                                font_size: FontSize::Px(10.0),
                                                ..default()
                                            },
                                            TextColor(Color::srgb(0.9, 0.9, 0.8)),
                                        ));
                                    });
                                }
                            });
                        }
                    });
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
