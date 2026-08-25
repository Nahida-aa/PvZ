use std::collections::HashMap;

use bevy::prelude::*;

use crate::plant::PlantKind;

/// 每种植物的冷却时间（秒），从 PlantKind 查询。
pub fn cooldown_duration(kind: PlantKind) -> f32 {
    match kind {
        PlantKind::Peashooter => 7.5,
        PlantKind::Sunflower => 5.0,
    }
}

#[derive(Resource)]
pub struct PlantCards {
    /// 各卡牌剩余冷却时间（秒），<= 0 表示可用。
    remaining: HashMap<PlantKind, f32>,
    /// 本关出战的卡牌列表（有序），用于 UI 渲染。
    pub slot_kinds: Vec<PlantKind>,
}

impl Default for PlantCards {
    fn default() -> Self {
        Self {
            remaining: HashMap::new(),
            slot_kinds: vec![],
        }
    }
}

impl PlantCards {
    /// 从关卡配置初始化：设定出战卡牌列表，所有冷却归零。
    pub fn init(&mut self, kinds: &[PlantKind]) {
        self.slot_kinds = kinds.to_vec();
        self.remaining.clear();
        for &k in kinds {
            self.remaining.entry(k).or_insert(0.0);
        }
    }

    pub fn remaining(&self, kind: PlantKind) -> f32 {
        self.remaining.get(&kind).copied().unwrap_or(0.0)
    }

    pub fn ready(&self, kind: &PlantKind) -> bool {
        self.remaining(*kind) <= 0.0
    }

    /// 种植后触发冷却。
    pub fn trigger(&mut self, kind: &PlantKind) {
        let duration = cooldown_duration(*kind);
        self.remaining.insert(*kind, duration);
    }

    /// 每帧扣减冷却时间。
    pub fn tick(&mut self, dt: f32) {
        for remaining in self.remaining.values_mut() {
            if *remaining > 0.0 {
                *remaining = (*remaining - dt).max(0.0);
            }
        }
    }
}
