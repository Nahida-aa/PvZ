use bevy::prelude::*;
use bevy::ui::prelude::{BorderRect, NodeImageMode, SliceScaleMode, TextureSlicer};
use bevy::ui::widget::TextShadow;
use bevy_ui_widgets::{Slider, SliderRange, SliderStep, SliderThumb, SliderValue, TrackClick, slider_self_update};

use super::components::*;
use crate::ui::menu::pause_menu::config::PauseMenuConfig;

/// 创建暂停菜单中的小按钮（"重新开始"、"主菜单"、"查看图鉴"、"选项设置"）。
pub(crate) fn spawn_small_button(
    parent: &mut ChildSpawnerCommands,
    btn_marker: impl Bundle,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    label: &str,
    font: &Handle<Font>,
    bg: &Handle<Image>,
    config: &PauseMenuConfig,
) {
    let sb = &config.small_button;
    let expand = &sb.bg_expand;
    let mins = &sb.bg_9slice_min;
    let maxs = &sb.bg_9slice_max;
    let bg_left = -expand[0];
    let bg_top = -expand[1];
    let bg_w = width + expand[0] + expand[2];
    let bg_h = height + expand[1] + expand[3];

    parent
        .spawn((
            Button,
            PauseButtonMarker,
            btn_marker,
            OriginalTop(top),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                width: Val::Px(width),
                height: Val::Px(height),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|b| {
            b.spawn((
                ImageNode {
                    image: bg.clone(),
                    image_mode: NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect {
                            min_inset: Vec2::new(mins[0], mins[1]),
                            max_inset: Vec2::new(maxs[0], maxs[1]),
                        },
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: 100.0,
                    }),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(bg_left),
                    top: Val::Px(bg_top),
                    width: Val::Px(bg_w),
                    height: Val::Px(bg_h),
                    ..default()
                },
            ));
            b.spawn((
                Text::new(label),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(sb.font_size),
                    ..default()
                },
                TextColor(Color::srgb(sb.text_color[0], sb.text_color[1], sb.text_color[2])),
            ));
        });
}

/// 创建滑动条（使用 bevy_ui_widgets::Slider）。
///
/// 布局：一行内左边 Label + 右边滑动条（track + knob）。
/// 返回 slider_entity（带 Slider 组件）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_slider(
    parent: &mut ChildSpawnerCommands,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    label: &str,
    slider_type: SliderType,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    font: &Handle<Font>,
    slot_image: &Handle<Image>,
    knob_image: &Handle<Image>,
    config: &PauseMenuConfig,
    label_area_width: Option<f32>,
) -> Entity {
    let sc = &config.slider;
    let label_area_width = label_area_width.unwrap_or(sc.label_area_width);
    let track_width = width - label_area_width;
    let track_height = sc.track_height;
    let knob_width = sc.knob_width;
    let knob_height = sc.knob_height;

    let mut root = parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(left),
            top: Val::Px(top),
            width: Val::Px(width),
            height: Val::Px(height),
            ..default()
        },
    ));

    let mut slider_entity = Entity::PLACEHOLDER;

    root.with_children(|row| {
        // Label
        row.spawn((
            Text::new(label),
            TextFont {
                font: FontSource::Handle(font.clone()),
                font_size: FontSize::Px(sc.label_font_size),
                ..default()
            },
            TextColor(Color::srgb(sc.label_color[0], sc.label_color[1], sc.label_color[2])),
            TextShadow {
                offset: Vec2::new(sc.label_shadow_offset[0], sc.label_shadow_offset[1]),
                color: Color::BLACK,
            },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(sc.label_top_offset),
                ..default()
            },
        ));

        // 滑动条轨道
        let ratio = ((value - min) / (max - min)).clamp(0.0, 1.0);

        let mut track = row.spawn((
            Slider {
                track_click: TrackClick::Snap,
                ..default()
            },
            SliderValue(value),
            SliderRange::new(min, max),
            SliderStep(step),
            slider_type,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(label_area_width),
                top: Val::Px(sc.track_top_offset),
                width: Val::Px(track_width),
                height: Val::Px(track_height),
                ..default()
            },
            ImageNode {
                image: slot_image.clone(),
                image_mode: NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect {
                        min_inset: Vec2::new(0.0, 5.0),
                        max_inset: Vec2::new(0.0, 5.0),
                    },
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 100.0,
                }),
                ..default()
            },
        ));

        track.observe(slider_self_update);

        slider_entity = track.id();

        track.with_children(|track_children| {
            track_children.spawn((
                SliderThumb,
                ImageNode::new(knob_image.clone()),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(ratio * (track_width - knob_width)),
                    top: Val::Px((track_height - knob_height) * 0.5),
                    width: Val::Px(knob_width),
                    height: Val::Px(knob_height),
                    ..default()
                },
            ));
        });
    });

    slider_entity
}
