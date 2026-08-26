use bevy::prelude::*;
use bevy::text::FontSize;
use crate::assets::GameAssets;
use crate::settings::CardConfig;
use crate::plant::PlantKind;

/// 卡片渲染的公共参数
pub struct CardVisualParams {
    pub width: f32,
    pub height: f32,
    pub font: Handle<Font>,
}

impl Default for CardVisualParams {
    fn default() -> Self {
        Self {
            width: 50.0,
            height: 70.0,
            font: Handle::default(),
        }
    }
}

/// 根据植物名称获取卡片图片
pub fn card_image_handle(plant: &str, assets: &GameAssets) -> Handle<Image> {
    match plant {
        "Peashooter" | "peashooter" => assets.card_peashooter.clone(),
        "Sunflower" | "sunflower" => assets.card_sunflower.clone(),
        _ => assets.card_peashooter.clone(),
    }
}

/// 根据 PlantKind 获取卡片图片
pub fn card_image_handle_kind(kind: PlantKind, assets: &GameAssets) -> Handle<Image> {
    match kind {
        PlantKind::Peashooter => assets.card_peashooter.clone(),
        PlantKind::Sunflower => assets.card_sunflower.clone(),
    }
}

/// 获取植物阳光消耗
pub fn plant_sun_cost(plant: &str) -> u32 {
    match plant {
        "Peashooter" | "peashooter" => 100,
        "Sunflower" | "sunflower" => 50,
        "CherryBomb" | "cherrybomb" => 150,
        "WallNut" | "wallnut" => 50,
        "PotatoMine" | "potatomine" => 25,
        "SnowPea" | "snowpea" => 175,
        "Chomper" | "chomper" => 150,
        "RepeaterPea" | "repeaterpea" => 200,
        "PuffShroom" | "puffshroom" => 0,
        "SunShroom" | "sunshroom" => 25,
        "FumeShroom" | "fumeshroom" => 75,
        "GraveBuster" | "gravebuster" => 75,
        "HypnoShroom" | "hypnoshroom" => 75,
        "IceShroom" | "iceshroom" => 75,
        "DoomShroom" | "doomshroom" => 125,
        "LilyPad" | "lilypad" => 25,
        "Squash" | "squash" => 50,
        "ThreePeaShooter" | "threepeashooter" => 325,
        "TangleKlep" | "tangleklep" => 25,
        "Jalapeno" | "jalapeno" => 125,
        "Spikeweed" | "spikeweed" => 100,
        "TorchWood" | "torchwood" => 175,
        "TallNut" | "tallnut" => 125,
        "Seashroom" | "seashroom" => 0,
        "Plantern" | "plantern" => 25,
        "Cactus" | "cactus" => 125,
        "Blover" | "blover" => 100,
        "StarFruit" | "starfruit" => 125,
        "PumpkinHead" | "pumpkinhead" => 125,
        "Garlic" | "garlic" => 50,
        "GiantWallNut" | "giantwallnut" => 125,
        _ => 0,
    }
}

/// 在 parent 下生成一张卡片：背景 + 图片 + 阳光消耗标签
pub fn spawn_card<'a>(
    parent: &'a mut ChildSpawnerCommands,
    plant: &str,
    assets: &GameAssets,
    cc: &CardConfig,
    x: f32,
    y: f32,
) -> EntityCommands<'a> {
    let card_bg = assets.seed_packet_larger.clone();
    let card_img = card_image_handle(plant, assets);
    let cost = plant_sun_cost(plant);
    let w = cc.card_slot_width;
    let h = cc.card_slot_height;

    let mut entity = parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(x),
            top: Val::Px(y),
            width: Val::Px(w),
            height: Val::Px(h),
            ..default()
        },
        ImageNode::new(card_bg),
    ));

    entity.with_children(|card| {
        // 卡片图片
        card.spawn((
            ImageNode::new(card_img),
            Node {
                width: Val::Px(w),
                height: Val::Px(h),
                ..default()
            },
        ));

        // 阳光消耗标签（底部）
        card.spawn((
            Text::new(format!("{cost}")),
            TextFont {
                font: FontSource::Handle(assets.font.clone()),
                font_size: FontSize::Px(12.0),
                ..default()
            },
            TextColor(Color::srgb(0.0, 0.0, 0.0)),
            TextLayout::justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(cc.card_cost_bottom),
                left: Val::Px(cc.card_cost_left),
                width: Val::Px(cc.card_cost_width),
                padding: UiRect::top(Val::Px(cc.card_cost_padding_top)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
        ));
    });

    entity
}
