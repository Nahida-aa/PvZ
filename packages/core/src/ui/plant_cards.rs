use bevy::prelude::*;

use crate::plant::PlantKind;

#[derive(Resource, Default)]
pub struct PlantCards {
    pub peashooter_remaining: f32,
    pub sunflower_remaining: f32,
}

impl PlantCards {
    pub fn remaining(&self, kind: PlantKind) -> f32 {
        match kind {
            PlantKind::Peashooter => self.peashooter_remaining,
            PlantKind::Sunflower => self.sunflower_remaining,
        }
    }

    pub fn ready(&self, kind: &PlantKind) -> bool {
        self.remaining(*kind) <= 0.0
    }

    pub fn trigger(&mut self, kind: &PlantKind) {
        let duration = match kind {
            PlantKind::Peashooter => 7.5,
            PlantKind::Sunflower => 5.0,
        };
        match kind {
            PlantKind::Peashooter => self.peashooter_remaining = duration,
            PlantKind::Sunflower => self.sunflower_remaining = duration,
        }
    }
}
