use bevy::prelude::*;
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};

use crate::assets::GameAssets;
use crate::combat::ApplyDamage;
use crate::lawn::{CELL_WIDTH, WIN_W};
use crate::schedule::GameSet;

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub speed: f32,
}

#[derive(Message)]
pub struct SpawnProjectile {
    pub pos: Vec3,
    pub damage: f32,
    pub speed: f32,
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnProjectile>()
            .add_systems(Update, handle_spawn_projectile.in_set(GameSet::Spawn))
            .add_systems(Update, move_projectiles.in_set(GameSet::Movement))
            .add_systems(Update, projectile_zombie_collision.in_set(GameSet::Combat))
            .add_systems(Update, cleanup_offscreen_projectiles.in_set(GameSet::Cleanup));
    }
}

fn handle_spawn_projectile(
    mut events: MessageReader<SpawnProjectile>,
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    for ev in events.read() {
        commands.spawn((
            Projectile {
                damage: ev.damage,
                speed: ev.speed,
            },
            Sprite::from_image(assets.pea_normal.clone()),
            Transform::from_translation(ev.pos),
            Visibility::default(),
            AudioPlayer::<AudioSource>(assets.shoot_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn move_projectiles(mut query: Query<(&mut Transform, &Projectile)>) {
    for (mut transform, projectile) in query.iter_mut() {
        transform.translation.x += projectile.speed;
    }
}

fn projectile_zombie_collision(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &Projectile)>,
    zombies: Query<(Entity, &Transform), With<crate::zombie::Zombie>>,
    mut damage_writer: MessageWriter<ApplyDamage>,
) {
    for (proj_entity, proj_transform, projectile) in projectiles.iter() {
        let proj_pos = proj_transform.translation.truncate();
        for (zombie_entity, zombie_transform) in zombies.iter() {
            let zombie_pos = zombie_transform.translation.truncate();
            if proj_pos.distance(zombie_pos) < 30.0 {
                damage_writer.write(ApplyDamage {
                    target: zombie_entity,
                    amount: projectile.damage,
                });
                commands.entity(proj_entity).despawn();
                break;
            }
        }
    }
}

fn cleanup_offscreen_projectiles(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform), With<Projectile>>,
) {
    let threshold = WIN_W / 2.0 + CELL_WIDTH;
    for (entity, transform) in projectiles.iter() {
        if transform.translation.x > threshold {
            commands.entity(entity).despawn();
        }
    }
}
