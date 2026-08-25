//! 外部化配置：app 级与关卡级视觉/布局参数。
//!
//! 没有可视化编辑器，视觉参数（窗口尺寸、背景摆位、草坪几何、割草机属性等）
//! 需要频繁微调。这些数据从 RON 文件读取，改文件即可生效、无需重新编译。
//!
//! 分层：
//! - [`AppConfig`]：app 级全局视觉（窗口尺寸、背景摆位），与具体关卡无关。
//! - [`LevelDefinition`]：关卡级布局（草坪几何、割草机属性、失败线），随关卡变化。

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
    /// 卡槽位置和大小。
    pub card_slot_top: f32,
    pub card_slot_width: f32,
    pub card_slot_height: f32,
    /// 卡牌费用文字位置：相对卡槽的 bottom, left。
    pub card_cost_bottom: f32,
    pub card_cost_left: f32,
    /// 卡牌费用文字区域宽度（数字在此宽度内水平居中）。
    pub card_cost_width: f32,
    /// 卡牌费用文字顶部内边距。
    pub card_cost_padding_top: f32,
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
            card_slot_top: 8.0,
            card_slot_width: 50.0,
            card_slot_height: 70.0,
            card_cost_bottom: 2.0,
            card_cost_left: 0.0,
            card_cost_width: 30.0,
            card_cost_padding_top: 0.0,
        }
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

/// 关卡级布局配置，从 `levels/level_XX.ron` 读取。
#[derive(Resource, Deserialize, Clone, Debug)]
pub struct LevelDefinition {
    /// 草坪网格几何。
    pub grid: GridConfig,
    /// 割草机属性。
    pub mower: MowerConfig,
    /// 房子碰撞箱中心 X（屏幕坐标），僵尸越过即失败。
    pub defeat_screen_x: f32,
    /// 出战卡槽最大数量（对齐 Godot max_choosed_card_num，默认 10）。
    #[serde(default = "default_max_choosed_card_num")]
    pub max_choosed_card_num: u32,
    /// 本关可用的卡牌列表（植物类型），未填满则剩余槽位显示占位图。
    #[serde(default)]
    pub card_kinds: Vec<String>,
}

fn default_max_choosed_card_num() -> u32 { 10 }

/// 草坪网格几何。
#[derive(Deserialize, Clone, Copy, Debug)]
pub struct GridConfig {
    /// 列数。
    pub cols: u32,
    /// 行数。
    pub rows: u32,
    /// 单元格宽度。
    pub cell_w: f32,
    /// 单元格高度。
    pub cell_h: f32,
    /// 网格在屏幕坐标系中的 X 轴起点。
    pub origin_x: f32,
    /// 网格在屏幕坐标系中的 Y 轴起点。
    pub origin_y: f32,
}

/// 割草机属性。
#[derive(Deserialize, Clone, Copy, Debug)]
pub struct MowerConfig {
    /// 移动速度（屏幕像素/帧，基于原 const 语义）。
    pub speed: f32,
    /// 触发范围：僵尸进入此距离即启动。
    pub trigger_range: f32,
    /// 击杀范围：覆盖此宽度的僵尸被清除。
    pub kill_range: f32,
    /// 碰撞箱宽度。
    pub hit_w: f32,
    /// 碰撞箱高度。
    pub hit_h: f32,
    /// 碰撞箱 Y 偏移。
    pub y_offset: f32,
    /// 初始屏幕 X（草坪左侧，紧挨第一列）。
    pub screen_x: f32,
}

impl LevelDefinition {
    /// 从 RON 文件加载关卡配置。
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let text =
            std::fs::read_to_string(path).map_err(|e| format!("读取 {path} 失败: {e}"))?;
        ron::from_str(&text).map_err(|e| format!("解析 {path} 失败: {e}"))
    }
}

impl Default for LevelDefinition {
    fn default() -> Self {
        Self {
            grid: GridConfig {
                cols: 9,
                rows: 5,
                cell_w: 80.0,
                cell_h: 99.0,
                origin_x: 195.0,
                origin_y: 80.0,
            },
            mower: MowerConfig {
                speed: 9.0,
                trigger_range: 17.0,
                kill_range: 50.0,
                hit_w: 33.5,
                hit_h: 48.0,
                y_offset: 17.0,
                screen_x: 171.0,
            },
            defeat_screen_x: 95.0,
            max_choosed_card_num: 10,
            card_kinds: vec![],
        }
    }
}

// ── 暂停菜单布局配置 ──────────────────────────────────────────────

/// 暂停菜单布局配置，从 `ui/pause_menu.ron` 读取。
///
/// 所有视觉参数（位置、尺寸、字体大小、颜色）均可在此文件中调整，
/// 无需重新编译。修改 .ron 文件后，Paused 状态下会自动热重载。
#[derive(Asset, Reflect, Deserialize, Clone, Debug)]
pub struct PauseMenuConfig {
    pub overlay: OverlayConfig,
    pub panel: PanelConfig,
    pub music_slider: SliderElement,
    pub sound_slider: SliderElement,
    pub speed_slider: SliderElement,
    pub speed_label: LabelElement,
    pub almanac_button: ButtonElement,
    pub options_button: ButtonElement,
    pub restart_button: ButtonElement,
    pub main_menu_button: ButtonElement,
    pub continue_button: ContinueButtonElement,
    pub small_button: SmallButtonConfig,
    pub slider: SliderConfig,
}

#[derive(Reflect, Deserialize, Clone, Debug)]
pub struct OverlayConfig {
    pub color: [f32; 4],
    pub z_index: i32,
}

#[derive(Reflect, Deserialize, Clone, Debug)]
pub struct PanelConfig {
    pub width: f32,
    pub height: f32,
}

#[derive(Reflect, Deserialize, Clone, Debug)]
pub struct SliderElement {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Reflect, Deserialize, Clone, Debug)]
pub struct LabelElement {
    pub left: f32,
    pub top: f32,
    pub font_size: f32,
    pub color: [f32; 3],
}

#[derive(Reflect, Deserialize, Clone, Debug)]
pub struct ButtonElement {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Reflect, Deserialize, Clone, Debug)]
pub struct ContinueButtonElement {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub text_color: [f32; 3],
}

#[derive(Reflect, Deserialize, Clone, Debug)]
pub struct SmallButtonConfig {
    pub font_size: f32,
    pub text_color: [f32; 3],
    /// expand_margin: [left, top, right, bottom]
    pub bg_expand: [f32; 4],
    pub bg_9slice_min: [f32; 2],
    pub bg_9slice_max: [f32; 2],
}

#[derive(Reflect, Deserialize, Clone, Debug)]
pub struct SliderConfig {
    pub label_font_size: f32,
    pub label_color: [f32; 3],
    pub track_height: f32,
    pub knob_width: f32,
    pub knob_height: f32,
    /// Label 区域宽度（滑动条左侧留白）。
    pub label_area_width: f32,
    /// Label 垂直偏移（相对滑动条根节点 top）。
    pub label_top_offset: f32,
    /// Track 垂直偏移（相对滑动条根节点 top）。
    pub track_top_offset: f32,
}

impl Default for PauseMenuConfig {
    fn default() -> Self {
        Self {
            overlay: OverlayConfig {
                color: [0.0, 0.0, 0.0, 0.45],
                z_index: 1000,
            },
            panel: PanelConfig {
                width: 412.0,
                height: 483.0,
            },
            music_slider: SliderElement {
                left: 97.0,
                top: 112.5,
                width: 218.0,
                height: 30.0,
            },
            sound_slider: SliderElement {
                left: 97.0,
                top: 152.5,
                width: 218.0,
                height: 30.0,
            },
            speed_slider: SliderElement {
                left: 97.0,
                top: 192.5,
                width: 218.0,
                height: 30.0,
            },
            speed_label: LabelElement {
                left: 175.0,
                top: 200.5,
                font_size: 14.0,
                color: [0.38, 0.384, 0.502],
            },
            almanac_button: ButtonElement {
                left: 103.0,
                top: 236.0,
                width: 88.0,
                height: 37.0,
            },
            options_button: ButtonElement {
                left: 210.0,
                top: 236.5,
                width: 88.0,
                height: 37.0,
            },
            restart_button: ButtonElement {
                left: 102.0,
                top: 279.5,
                width: 195.0,
                height: 37.0,
            },
            main_menu_button: ButtonElement {
                left: 102.0,
                top: 323.5,
                width: 195.0,
                height: 37.0,
            },
            continue_button: ContinueButtonElement {
                left: 26.0,
                top: 383.0,
                width: 360.0,
                height: 100.0,
                font_size: 50.0,
                text_color: [0.0, 1.0, 0.0],
            },
            small_button: SmallButtonConfig {
                font_size: 18.0,
                text_color: [0.0, 1.0, 0.0],
                bg_expand: [8.0, 5.0, 5.0, 8.0],
                bg_9slice_min: [16.0, 16.0],
                bg_9slice_max: [16.0, 20.0],
            },
            slider: SliderConfig {
                label_font_size: 14.0,
                label_color: [0.38, 0.384, 0.502],
                track_height: 10.0,
                knob_width: 22.0,
                knob_height: 29.0,
                label_area_width: 62.0,
                label_top_offset: 8.0,
                track_top_offset: 10.0,
            },
        }
    }
}
