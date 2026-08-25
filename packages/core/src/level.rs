use bevy::prelude::*;
use serde::Deserialize;

use crate::schedule::GameSet;
use crate::state::GameState;
use crate::zombie::SpawnZombie;

struct WaveEntry {
    start_after: f32,
    count: u32,
    interval: f32,
    rows: core::ops::Range<u32>,
}

const WAVES: &[WaveEntry] = &[
    WaveEntry { start_after: 5.0, count: 3, interval: 3.0, rows: 0..3 },
    WaveEntry { start_after: 25.0, count: 5, interval: 2.5, rows: 1..4 },
    WaveEntry { start_after: 50.0, count: 8, interval: 2.0, rows: 0..5 },
    WaveEntry { start_after: 80.0, count: 10, interval: 1.5, rows: 0..5 },
];

#[derive(Resource)]
pub struct LevelRuntime {
    elapsed: f32,
    wave_index: usize,
    wave_timer: f32,
    remaining: u32,
    rows: core::ops::Range<u32>,
    spawn_interval: f32,
    active: bool,
}

impl Default for LevelRuntime {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            wave_index: 0,
            wave_timer: 0.0,
            remaining: 0,
            rows: 0..0,
            spawn_interval: 1.0,
            active: false,
        }
    }
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelRuntime>()
            .add_systems(
                Update,
                tick_wave_timeline
                    .in_set(GameSet::Spawn)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn tick_wave_timeline(
    time: Res<Time>,
    mut runtime: ResMut<LevelRuntime>,
    mut spawner: MessageWriter<SpawnZombie>,
) {
    runtime.elapsed += time.delta_secs();

    if !runtime.active {
        if runtime.wave_index < WAVES.len()
            && runtime.elapsed >= WAVES[runtime.wave_index].start_after
        {
            let wave = &WAVES[runtime.wave_index];
            runtime.remaining = wave.count;
            runtime.rows = wave.rows.clone();
            runtime.spawn_interval = wave.interval;
            runtime.wave_timer = 0.0;
            runtime.active = true;
        }
        return;
    }

    runtime.wave_timer += time.delta_secs();
    if runtime.wave_timer >= runtime.spawn_interval && runtime.remaining > 0 {
        runtime.wave_timer = 0.0;
        runtime.remaining -= 1;
        let row_count = runtime.rows.end - runtime.rows.start;
        let row = if row_count > 0 {
            runtime.rows.start + runtime.remaining % row_count
        } else {
            0
        };
        spawner.write(SpawnZombie { row });
    }

    if runtime.remaining == 0 {
        runtime.active = false;
        runtime.wave_index += 1;
    }
}

// ── 关卡定义 ──────────────────────────────────────────────────────

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
    pub cols: u32,
    pub rows: u32,
    pub cell_w: f32,
    pub cell_h: f32,
    pub origin_x: f32,
    pub origin_y: f32,
}

/// 割草机属性。
#[derive(Deserialize, Clone, Copy, Debug)]
pub struct MowerConfig {
    pub speed: f32,
    pub trigger_range: f32,
    pub kill_range: f32,
    pub hit_w: f32,
    pub hit_h: f32,
    pub y_offset: f32,
    pub screen_x: f32,
}

impl LevelDefinition {
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
