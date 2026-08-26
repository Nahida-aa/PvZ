//! App 级全局设置：窗口尺寸、背景摆位、音量、菜单栏布局。
//!
//! 从 `app.ron` 读取。与具体关卡无关。

use bevy::prelude::*;
use serde::Deserialize;

/// app 级全局配置，从 `app.ron` 读取。
#[derive(Resource, Deserialize, Clone, Debug)]
pub struct AppConfig {
    /// 窗口尺寸 (宽, 高)，单位像素。
    pub window: (f32, f32),
    /// 背景图摆位。
    pub bg: BgConfig,
    /// 主音量 (0.0 ~ 1.0)。
    #[serde(default = "default_master_volume")]
    pub master_volume: f32,
    /// 背景音乐音量 (0.0 ~ 1.0)。
    #[serde(default = "default_bgm_volume")]
    pub bgm_volume: f32,
    /// 音效音量 (0.0 ~ 1.0)。
    #[serde(default = "default_sfx_volume")]
    pub sfx_volume: f32,
    /// 游戏倍速 (0.5 ~ 3.0)。
    #[serde(default = "default_time_scale")]
    pub time_scale: f32,
    /// 顶部菜单栏布局参数。
    #[serde(default)]
    pub menubar: MenubarConfig,
}

fn default_master_volume() -> f32 { 1.0 }
fn default_bgm_volume() -> f32 { 0.5 }
fn default_sfx_volume() -> f32 { 0.5 }
fn default_time_scale() -> f32 { 1.0 }

/// 背景图布局参数。
#[derive(Deserialize, Clone, Copy, Debug)]
pub struct BgConfig {
    /// 背景图宽度。
    pub img_w: f32,
    /// 背景图高度。
    pub img_h: f32,
    /// 背景视口对齐偏移：值越大背景越靠左，越小越靠右。
    pub viewport_x: f32,
}

/// 顶部菜单栏布局参数。
#[derive(Deserialize, Clone, Copy, Debug)]
pub struct MenubarConfig {
    /// 菜单栏背景左偏移。
    pub bar_left: f32,
    /// 菜单栏背景顶偏移。
    pub bar_top: f32,
    /// 阳光文字位置：相对菜单栏背景的 left, bottom。
    pub sun_counter_left: f32,
    pub sun_counter_bottom: f32,
    /// 阳光文字容器大小。
    pub sun_counter_width: f32,
    pub sun_counter_height: f32,
    /// 阳光文字顶部内边距（补偿 Bevy 文字从 top-left 渲染的问题）。
    pub sun_counter_padding_top: f32,
    /// 阳光飘入目标点：相对菜单栏背景的 left, top。
    pub sun_target_left: f32,
    pub sun_target_top: f32,
}

impl Default for MenubarConfig {
    fn default() -> Self {
        Self {
            bar_left: 150.0,
            bar_top: 0.0,
            sun_counter_left: 10.0,
            sun_counter_bottom: 6.0,
            sun_counter_width: 58.0,
            sun_counter_height: 28.0,
            sun_counter_padding_top: 0.0,
            sun_target_left: 39.0,
            sun_target_top: 26.0,
        }
    }
}

/// 卡片布局配置，从 `ui/card.jsonc` 读取。
#[derive(Resource, Deserialize, Clone, Copy, Debug)]
pub struct CardConfig {
    /// 卡槽相对菜单栏顶部偏移。
    pub card_slot_top: f32,
    /// 卡槽宽度。
    pub card_slot_width: f32,
    /// 卡槽高度。
    pub card_slot_height: f32,
    /// 阳光消耗文字位置。
    pub card_cost_bottom: f32,
    pub card_cost_left: f32,
    pub card_cost_width: f32,
    pub card_cost_padding_top: f32,
    /// 待选面板布局。
    pub candidate_cols: usize,
    pub candidate_rows: usize,
    pub candidate_card_gap_x: f32,
    pub candidate_card_gap_y: f32,
    pub candidate_offset_x: f32,
    pub candidate_offset_y: f32,
}

impl Default for CardConfig {
    fn default() -> Self {
        Self {
            card_slot_top: 8.0,
            card_slot_width: 50.0,
            card_slot_height: 70.0,
            card_cost_bottom: 0.0,
            card_cost_left: 0.0,
            card_cost_width: 35.0,
            card_cost_padding_top: 0.0,
            candidate_cols: 8,
            candidate_rows: 6,
            candidate_card_gap_x: 5.0,
            candidate_card_gap_y: 2.0,
            candidate_offset_x: 13.0,
            candidate_offset_y: 32.0,
        }
    }
}

impl CardConfig {
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let text =
            std::fs::read_to_string(path).map_err(|e| format!("读取 {path} 失败: {e}"))?;
        let value = jsonc_parser::parse_to_serde_value(&text, &Default::default())
            .map_err(|e| format!("解析 {path} 失败: {e}"))?
            .ok_or_else(|| format!("{path} 内容为空"))?;
        serde_json::from_value(value).map_err(|e| format!("反序列化 {path} 失败: {e}"))
    }
}

impl AppConfig {
    /// 窗口宽度。
    pub fn win_w(&self) -> f32 {
        self.window.0
    }

    /// 窗口高度。
    pub fn win_h(&self) -> f32 {
        self.window.1
    }

    /// 从 RON 文件加载 app 配置。
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let text =
            std::fs::read_to_string(path).map_err(|e| format!("读取 {path} 失败: {e}"))?;
        ron::from_str(&text).map_err(|e| format!("解析 {path} 失败: {e}"))
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: (1066.0, 600.0),
            bg: BgConfig {
                img_w: 1400.0,
                img_h: 600.0,
                viewport_x: 60.0,
            },
            master_volume: 1.0,
            bgm_volume: 0.5,
            sfx_volume: 0.5,
            time_scale: 1.0,
            menubar: MenubarConfig::default(),
        }
    }
}
