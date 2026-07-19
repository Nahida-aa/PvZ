use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::animation::SpriteAnimation;
use crate::assets::GameAssets;
use crate::combat::{DeathCleanup, Health, Team};
use crate::lawn::GridPos;
use crate::projectile::SpawnProjectile;
use crate::schedule::GameSet;

#[derive(Component)]
pub struct Plant {
    pub kind: PlantKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlantKind {
    Peashooter,
    Sunflower,
}

impl PlantKind {
    pub fn cost(&self) -> u32 {
        match self {
            PlantKind::Peashooter => 100,
            PlantKind::Sunflower => 50,
        }
    }
}

#[derive(Component)]
pub struct Shooter {
    pub cooldown: f32,
    pub timer: f32,
}

#[derive(Component)]
pub struct SunProducer {
    pub interval: f32,
    pub timer: f32,
}

#[derive(Message)]
pub struct SpawnPlant {
    pub kind: PlantKind,
    pub pos: GridPos,
}

pub struct PlantPlugin;

impl Plugin for PlantPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnPlant>()
            .add_systems(Update, handle_spawn_plant.in_set(GameSet::Spawn))
            .add_systems(Update, shooter_fire.in_set(GameSet::Movement))
            .add_systems(Update, sun_producer_tick.in_set(GameSet::Movement));
    }
}

fn handle_spawn_plant(
    mut events: MessageReader<SpawnPlant>,
    mut commands: Commands,
    mut occupancy: ResMut<crate::lawn::LawnOccupancy>,
    assets: Res<GameAssets>,
) {
    for ev in events.read() {
        if !occupancy.is_free(ev.pos) {
            continue;
        }
        occupancy.occupy(ev.pos);

        let frames = match ev.kind {
            PlantKind::Peashooter => assets.peashooter_frames.clone(),
            PlantKind::Sunflower => assets.sunflower_frames.clone(),
        };

        let mut entity = commands.spawn((
            Plant { kind: ev.kind },
            Health::new(100.0),
            Team::Plant,
            DeathCleanup,
            ev.pos,
            Sprite::from_image(frames[0].clone()),
            Anchor::BOTTOM_CENTER,
            SpriteAnimation {
                frames,
                frame_duration: 0.12,
                timer: 0.0,
                current: 0,
            },
            Transform::from_translation(ev.pos.world_bottom().extend(1.0)),
            Visibility::default(),
        ));

        match ev.kind {
            PlantKind::Peashooter => {
                entity.insert(Shooter { cooldown: 1.5, timer: 0.0 });
            }
            PlantKind::Sunflower => {
                entity.insert(SunProducer { interval: 8.0, timer: 0.0 });
            }
        }
    }
}

fn shooter_fire(
    time: Res<Time>,
    mut query: Query<(&mut Shooter, &Transform)>,
    mut spawner: MessageWriter<SpawnProjectile>,
) {
    for (mut shooter, transform) in query.iter_mut() {
        shooter.timer += time.delta_secs();
        if shooter.timer >= shooter.cooldown {
            shooter.timer = 0.0;
            spawner.write(SpawnProjectile {
                pos: Vec3::new(
                    transform.translation.x + 40.0,
                    transform.translation.y + 40.0,
                    transform.translation.z + 1.0,
                ),
                damage: 20.0,
                speed: 5.0,
            });
        }
    }
}

fn sun_producer_tick(
    time: Res<Time>,
    mut query: Query<&mut SunProducer>,
    mut bank: ResMut<crate::components::menebar::SunBank>,
) {
    for mut producer in query.iter_mut() {
        producer.timer += time.delta_secs();
        if producer.timer >= producer.interval {
            producer.timer = 0.0;
            bank.amount += 25;
        }
    }
}
