use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum Team {
    Plant,
    Zombie,
}

#[derive(Message)]
pub struct ApplyDamage {
    pub target: Entity,
    pub amount: f32,
}

#[derive(Message)]
pub struct EntityDied(pub Entity);

#[derive(Component)]
pub struct DeathCleanup;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ApplyDamage>()
            .add_message::<EntityDied>()
            .add_systems(
                Update,
                (apply_damage, cleanup_dead).in_set(crate::schedule::GameSet::Combat),
            );
    }
}

fn apply_damage(
    mut events: MessageReader<ApplyDamage>,
    mut query: Query<&mut Health>,
    mut died: MessageWriter<EntityDied>,
) {
    for ev in events.read() {
        if let Ok(mut health) = query.get_mut(ev.target) {
            health.current -= ev.amount;
            if health.current <= 0.0 {
                died.write(EntityDied(ev.target));
            }
        }
    }
}

fn cleanup_dead(
    mut commands: Commands,
    mut died: MessageReader<EntityDied>,
    query: Query<Has<DeathCleanup>>,
) {
    for ev in died.read() {
        if let Ok(has_cleanup) = query.get(ev.0) {
            if has_cleanup {
                commands.entity(ev.0).despawn();
            }
        }
    }
}
