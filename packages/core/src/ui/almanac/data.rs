use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::assets::GameAssets;

#[derive(Deserialize)]
pub struct LocalInfo {
    pub name: String,
    pub description: String,
    pub intro: String,
    pub hint: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct LocalData {
    pub sun_cost: Option<u32>,
    pub cooldown: Option<f64>,
    pub toughness: Option<u64>,
    pub damage: Option<u64>,
    pub interval: Option<f64>,
    pub bg: Option<String>,
    pub preview_image: Option<String>,
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

#[derive(bevy::prelude::Resource)]
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

pub fn dir_to_card_name(dir: &str) -> Option<&'static str> {
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

pub fn get_ground_bg(bg: &str, assets: &GameAssets) -> Handle<Image> {
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
