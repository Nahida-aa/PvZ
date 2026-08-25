use bevy::prelude::*;

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
