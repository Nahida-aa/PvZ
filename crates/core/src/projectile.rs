use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::combat::ApplyDamage;
use crate::lawn::{CELL_WIDTH, WIN_W};
use crate::schedule::GameSet;
use crate::zombie::ZombieCollider;

const HIT_ANIM_DURATION: f32 = 0.25;

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub speed: f32,
}

#[derive(Component)]
pub struct HitAnim {
    pub timer: f32,
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
            .add_systems(Update, hit_anim_tick.in_set(GameSet::Movement))
            .add_systems(
                Update,
                cleanup_offscreen_projectiles.in_set(GameSet::Cleanup),
            );
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
        ));
        commands.spawn((
            AudioPlayer::<AudioSource>(assets.shoot_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn move_projectiles(time: Res<Time>, mut query: Query<(&mut Transform, &mut Projectile)>) {
    for (mut transform, projectile) in query.iter_mut() {
        transform.translation.x += projectile.speed * time.delta_secs();
        transform.translation.z = 2.0;
    }
}

fn projectile_zombie_collision(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &Transform, &Projectile, &mut Sprite)>,
    zombies: Query<(Entity, &Transform, &ZombieCollider)>,
    mut damage_writer: MessageWriter<ApplyDamage>,
    assets: Res<GameAssets>,
) {
    for (proj_entity, proj_transform, projectile, mut sprite) in projectiles.iter_mut() {
        let proj_pos = proj_transform.translation.truncate();
        for (zombie_entity, zombie_transform, collider) in zombies.iter() {
            let zombie_center = zombie_transform.translation.truncate() + collider.center_offset;
            let half = collider.half_size;
            let proj_half: f32 = 8.0;

            let overlap_x = (half.x + proj_half) - (proj_pos.x - zombie_center.x).abs();
            let overlap_y = (half.y + proj_half) - (proj_pos.y - zombie_center.y).abs();

            if overlap_x > 0.0 && overlap_y > 0.0 {
                damage_writer.write(ApplyDamage {
                    target: zombie_entity,
                    amount: projectile.damage,
                });
                sprite.image = assets.pea_normal_explode.clone();
                commands.entity(proj_entity).insert(HitAnim { timer: 0.0 });
                commands.entity(proj_entity).remove::<Projectile>();
                commands.spawn((
                    AudioPlayer::<AudioSource>(assets.bullet_explode_sound.clone()),
                    PlaybackSettings::DESPAWN,
                ));
                break;
            }
        }
    }
}

fn hit_anim_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut HitAnim)>,
) {
    for (entity, mut anim) in query.iter_mut() {
        anim.timer += time.delta_secs();
        if anim.timer >= HIT_ANIM_DURATION {
            commands.entity(entity).despawn();
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
