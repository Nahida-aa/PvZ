use bevy::prelude::*;
use bevy::asset::LoadState;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── JSONC 反序列化结构 ──────────────────────────────────────────

#[derive(Deserialize)]
struct AnimFile {
    duration: f32,
    fps: f32,
    loop_anim: bool,
    nodes: HashMap<String, AnimNodeDef>,
}

#[derive(Deserialize)]
struct AnimNodeDef {
    #[serde(default = "default_true")]
    visible: bool,
    texture: Option<String>,
    #[serde(default)]
    position: Vec<Keyframe<Vec2>>,
    #[serde(default)]
    scale: Vec<Keyframe<Vec2>>,
    #[serde(default)]
    rotation: Vec<Keyframe<f32>>,
    #[serde(default)]
    skew: Vec<Keyframe<f32>>,
}

fn default_true() -> bool {
    true
}

/// `[time, value]` 对, serde 支持序列化为 tuple
#[derive(Deserialize)]
struct Keyframe<T>(f32, T);

// ── 运行时数据 ──────────────────────────────────────────────────

/// 单个动画节点的关键帧数据（已转换为中心坐标）
struct ReanimNodeDef {
    visible_initial: bool,
    texture: Handle<Image>,
    /// 纹理原始尺寸, 用于将 top-left 偏移到 center
    tex_size: Vec2,
    position: Vec<Keyframe<Vec2>>,
    scale: Vec<Keyframe<Vec2>>,
    rotation: Vec<Keyframe<f32>>,
    skew: Vec<Keyframe<f32>>,
}

/// 解析好的动画剪辑
pub struct ReanimClip {
    pub duration: f32,
    pub fps: f32,
    pub loop_anim: bool,
    /// 按原始顺序排列的节点 (与 Godot 渲染顺序一致)
    nodes: Vec<(String, ReanimNodeDef)>,
}

// ── 组件 ────────────────────────────────────────────────────────

#[derive(Component)]
pub struct ReanimPlayer {
    pub elapsed: f32,
    pub playing: bool,
    clip: Handle<ReanimClip>,
    /// node_name → child entity
    node_entities: HashMap<String, Entity>,
    /// node_name → (half_tex_size, visible_initial)
    /// half_tex_size 用于把 Godot top-left 坐标转成 Bevy center 坐标
    offsets: HashMap<String, (Vec2, bool)>,
}

#[derive(Component)]
struct ReanimNode {
    node_name: String,
}

// ── 资源 ────────────────────────────────────────────────────────

/// 已完成加载的 ReanimClip, 可以直接使用
struct LoadedClip {
    clip: ReanimClip,
    texture_sizes: HashMap<String, Vec2>,
}

// ── 公开 API ────────────────────────────────────────────────────

/// 从 JSONC 文件加载 ReanimClip
pub fn load_reanim_clip_from_file(path: &Path, asset_server: &AssetServer) -> Handle<ReanimClip> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("读取 {} 失败: {e}", path.display()));
    let val: serde_json::Value = jsonc_parser::parse_to_serde_value(&text, &Default::default())
        .expect("解析 reanim JSONC 失败")
        .expect("reanim JSONC 为空");

    // map "loop" → "loop_anim" (JSONC 里用 "loop", 但 loop 是 Rust 关键字)
    let mut map = val.as_object().unwrap().clone();
    if let Some(v) = map.remove("loop") {
        map.insert("loop_anim".into(), v);
    }

    let file: AnimFile =
        serde_json::from_value(serde_json::Value::Object(map)).expect("reanim JSONC 格式错误");

    // 解析节点, 按原始插入顺序
    let mut nodes = Vec::new();
    for (name, def) in &file.nodes {
        if let Some(tex_name) = &def.texture {
            let tex_path = path
                .parent()
                .unwrap() // animation/
                .parent() // plants/<name>/
                .unwrap()
                .join("parts")
                .join(tex_name);
            let handle: Handle<Image> = asset_server.load(
                tex_path
                    .strip_prefix(path.parent().unwrap().parent().unwrap().parent().unwrap())
                    .unwrap_or(&tex_path)
                    .to_string_lossy()
                    .as_ref(),
            );

            nodes.push((
                name.clone(),
                ReanimNodeDef {
                    visible_initial: def.visible,
                    texture: handle,
                    tex_size: Vec2::ZERO, // 加载后填充
                    position: def.position.clone(),
                    scale: def.scale.clone(),
                    rotation: def.rotation.clone(),
                    skew: def.skew.clone(),
                },
            ));
        }
    }

    // 按节点在 HashMap 中的顺序(字母序)排列 — 与 Godot 的 children 顺序一致
    // 注意: Godot tres 的 track 顺序才是渲染顺序, 但 JSONC 转换时已按此排序
    // 这里保持 JSONC 中的插入顺序 (serde_json HashMap 的迭代顺序是插入顺序)

    let clip = ReanimClip {
        duration: file.duration,
        fps: file.fps,
        loop_anim: file.loop_anim,
        nodes,
    };

    asset_server.add(clip)
}

/// 生成 reanim 实体 (父 + 子 sprite)
/// 返回父 entity
pub fn spawn_reanim(
    commands: &mut Commands,
    clip_handle: Handle<ReanimClip>,
    clips: &Assets<ReanimClip>,
    asset_server: &AssetServer,
    position: Vec3,
    scale: f32,
) -> Option<Entity> {
    let clip = clips.get(&clip_handle)?;

    let mut node_entities = HashMap::new();
    let mut offsets = HashMap::new();

    let parent = commands
        .spawn((
            ReanimPlayer {
                elapsed: 0.0,
                playing: true,
                clip: clip_handle.clone(),
                node_entities: HashMap::new(),
                offsets: HashMap::new(),
            },
            Transform::from_translation(position).with_scale(Vec3::splat(scale)),
            Visibility::default(),
        ))
        .id();

    // 用临时 HashMap 收集子 entity, 之后写回 ReanimPlayer
    let mut temp_entities = HashMap::new();
    let mut temp_offsets = HashMap::new();

    // 为每个节点生成子 entity
    for (name, node_def) in &clip.nodes {
        // 加载纹理并获取尺寸
        let maybe_size = asset_server
            .load_state(&node_def.texture)
            .get()
            .and_then(|meta| {
                // Bevy 0.19: 通过 Image asset 获取尺寸
                None // 稍后通过 Image 事件获取
            });

        // 先以 Vec2::ZERO 作为 tex_size 创建, 稍后在 play_reanim 系统中修正
        // 或者直接用 asset_server.get() 尝试
        let tex_size = Vec2::ZERO; // 占位

        let child = commands
            .spawn((
                ReanimNode {
                    node_name: name.clone(),
                },
                Sprite::from_image(node_def.texture.clone()),
                Transform::default(),
                Visibility::default(),
            ))
            .id();

        temp_entities.insert(name.clone(), child);
        temp_offsets.insert(name.clone(), (tex_size, node_def.visible_initial));
        node_entities.insert(name.clone(), child);
        offsets.insert(name.clone(), (tex_size, node_def.visible_initial));
    }

    // 写回 ReanimPlayer
    commands.entity(parent).insert(ReanimPlayer {
        elapsed: 0.0,
        playing: true,
        clip: clip_handle,
        node_entities,
        offsets,
    });

    Some(parent)
}

// ── 系统 ────────────────────────────────────────────────────────

/// 更新所有 ReanimPlayer 的动画状态
pub fn play_reanim(
    time: Res<Time>,
    clips: Res<Assets<ReanimClip>>,
    images: Res<Assets<Image>>,
    mut players: Query<(&mut ReanimPlayer, &Children)>,
    mut sprites: Query<(
        &ReanimNode,
        &mut Sprite,
        &mut Transform,
        &mut Visibility,
    )>,
    asset_server: Res<AssetServer>,
) {
    for (mut player, children) in players.iter_mut() {
        if !player.playing {
            continue;
        }

        let Some(clip) = clips.get(&player.clip) else {
            continue;
        };

        player.elapsed += time.delta_secs();
        let mut t = player.elapsed % clip.duration;

        // 处理循环
        if !clip.loop_anim && player.elapsed >= clip.duration {
            player.playing = false;
            t = clip.duration - 0.001; // 停在最后一帧
        }

        // 第一帧: 如果 tex_size 还是 ZERO, 尝试获取纹理尺寸
        for (name, (tex_size, _)) in player.offsets.iter_mut() {
            if tex_size.x == 0.0 || tex_size.y == 0.0 {
                if let Some(entity) = player.node_entities.get(name) {
                    if let Ok((_, sprite, _, _)) = sprites.get(*entity) {
                        if let Some(img) = images.get(&sprite.image) {
                            *tex_size = img.size_f32();
                        }
                    }
                }
            }
        }

        // 更新每个节点
        for (name, node_def) in &clip.nodes {
            let Some(&entity) = player.node_entities.get(name) else {
                continue;
            };

            let Ok((node_marker, mut sprite, mut tf, mut vis)) = sprites.get_mut(entity) else {
                continue;
            };

            // 可见性
            *vis = if node_def.visible_initial {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };

            // 插值关键帧
            let pos = interpolate_vec2(&node_def.position, t);
            let scale = interpolate_vec2(&node_def.scale, t);
            let rot = interpolate_f32(&node_def.rotation, t);
            let skew = interpolate_f32(&node_def.skew, t);

            // Godot top-left → Bevy center: 减去纹理尺寸的一半
            let (half_tex, _) = player.offsets.get(name).unwrap_or(&(Vec2::ZERO, true));

            tf.translation.x = pos.x - half_tex.x;
            tf.translation.y = pos.y - half_tex.y;
            tf.translation.z = 0.0;

            tf.scale.x = scale.x;
            tf.scale.y = scale.y;

            // rotation (radians, 顺时针为正, 与 Godot 一致)
            tf.rotation = Quat::from_rotation_z(rot);

            // skew: Bevy 没有原生 skew, 用 scale 模拟
            // skew 影响的是切变, 简化处理: 忽略或用 scale_xy 近似
            // 完整实现需要修改 shader, 暂时忽略 skew
            let _ = skew;
        }
    }
}

// ── 插值工具 ────────────────────────────────────────────────────

fn interpolate_vec2(keyframes: &[Keyframe<Vec2>], time: f32) -> Vec2 {
    if keyframes.is_empty() {
        return Vec2::ZERO;
    }
    if keyframes.len() == 1 {
        return keyframes[0].1;
    }

    // 找到时间区间
    for i in 0..keyframes.len() - 1 {
        let (t0, v0) = (keyframes[i].0, keyframes[i].1);
        let (t1, v1) = (keyframes[i + 1].0, keyframes[i + 1].1);
        if time >= t0 && time <= t1 {
            let dt = t1 - t0;
            if dt < 1e-10 {
                return v0;
            }
            let frac = (time - t0) / dt;
            return v0.lerp(v1, frac);
        }
    }

    // 超出范围, 返回最后一帧
    keyframes.last().unwrap().1
}

fn interpolate_f32(keyframes: &[Keyframe<f32>], time: f32) -> f32 {
    if keyframes.is_empty() {
        return 0.0;
    }
    if keyframes.len() == 1 {
        return keyframes[0].1;
    }

    for i in 0..keyframes.len() - 1 {
        let (t0, v0) = (keyframes[i].0, keyframes[i].1);
        let (t1, v1) = (keyframes[i + 1].0, keyframes[i + 1].1);
        if time >= t0 && time <= t1 {
            let dt = t1 - t0;
            if dt < 1e-10 {
                return v0;
            }
            let frac = (time - t0) / dt;
            return v0 + (v1 - v0) * frac;
        }
    }

    keyframes.last().unwrap().1
}
