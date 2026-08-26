//! 暂停菜单布局定义，从 `ui/pause_menu.ron` 读取。
//!
//! 所有视觉参数（位置、尺寸、字体大小、颜色）均可在此文件中调整，
//! 无需重新编译。修改 .ron 文件后，Paused 状态下会自动热重载。

use bevy::prelude::*;
use serde::Deserialize;

#[derive(Resource, Deserialize, Clone, Debug)]
#[serde(default)]
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

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct OverlayConfig {
    pub color: [f32; 4],
    pub z_index: i32,
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct PanelConfig {
    /// 背景图片显示宽度（像素）。None = 使用 PNG 原始宽度。
    pub width: Option<f32>,
    /// 背景图片显示高度（像素）。None = 使用 PNG 原始高度。
    pub height: Option<f32>,
    /// 背景图片路径（相对于 assets 目录）。
    pub background_image: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SliderElement {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    /// Label 区域宽度（滑动条左侧留白）。None = 使用全局 slider.label_area_width。
    pub label_area_width: Option<f32>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LabelElement {
    pub left: f32,
    pub top: f32,
    pub font_size: f32,
    pub color: [f32; 3],
}

#[derive(Deserialize, Clone, Debug)]
pub struct ButtonElement {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ContinueButtonElement {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub text_color: [f32; 3],
}

#[derive(Deserialize, Clone, Debug)]
pub struct SmallButtonConfig {
    pub font_size: f32,
    pub text_color: [f32; 3],
    /// expand_margin: [left, top, right, bottom]
    pub bg_expand: [f32; 4],
    pub bg_9slice_min: [f32; 2],
    pub bg_9slice_max: [f32; 2],
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct SliderConfig {
    pub label_font_size: f32,
    pub label_color: [f32; 3],
    /// 描边宽度（Godot Label2 outline_size=2）。
    pub label_outline_size: f32,
    /// 阴影偏移 (x, y)（Godot Label2 shadow_offset=(-2,-2)）。
    pub label_shadow_offset: [f32; 2],
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

impl Default for SliderConfig {
    fn default() -> Self {
        Self {
            label_font_size: 18.0,
            label_color: [0.38, 0.384, 0.502],
            label_outline_size: 2.0,
            label_shadow_offset: [-2.0, -2.0],
            track_height: 10.0,
            knob_width: 22.0,
            knob_height: 29.0,
            label_area_width: 62.0,
            label_top_offset: 8.0,
            track_top_offset: 10.0,
        }
    }
}

/// 读取 PNG 文件头部，返回 (width, height)。
///
/// PNG 格式：8 字节签名 + 4 字节 IHDR 长度 + 4 字节 "IHDR" + 4 字节 width + 4 字节 height。
pub fn png_dimensions(path: &str) -> Result<(u32, u32), String> {
    let data = std::fs::read(path).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    if data.len() < 24 || &data[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(format!("{path} 不是有效的 PNG 文件"));
    }
    let width = u32::from_be_bytes(data[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(data[20..24].try_into().unwrap());
    Ok((width, height))
}

impl PauseMenuConfig {
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("读取 {path} 失败: {e}"))?;
        let value = jsonc_parser::parse_to_serde_value(&text, &Default::default())
            .map_err(|e| format!("解析 {path} 失败: {e}"))?
            .ok_or_else(|| format!("{path} 内容为空"))?;
        serde_json::from_value(value).map_err(|e| format!("反序列化 {path} 失败: {e}"))
    }
}

impl Default for PauseMenuConfig {
    fn default() -> Self {
        Self {
            overlay: OverlayConfig {
                color: [0.0, 0.0, 0.0, 0.45],
                z_index: 1000,
            },
            panel: PanelConfig {
                width: None,
                height: None,
                background_image: "graphics/Screen/option_dialog.png".into(),
            },
            music_slider: SliderElement {
                left: 97.0,
                top: 112.5,
                width: 218.0,
                height: 30.0,
                label_area_width: None,
            },
            sound_slider: SliderElement {
                left: 97.0,
                top: 152.5,
                width: 218.0,
                height: 30.0,
                label_area_width: None,
            },
            speed_slider: SliderElement {
                left: 97.0,
                top: 192.5,
                width: 218.0,
                height: 30.0,
                label_area_width: None,
            },
            speed_label: LabelElement {
                left: 175.0,
                top: 200.5,
                font_size: 18.0,
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
                label_font_size: 18.0,
                label_color: [0.38, 0.384, 0.502],
                label_outline_size: 2.0,
                label_shadow_offset: [-2.0, -2.0],
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
