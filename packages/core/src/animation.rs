use bevy::prelude::*;

#[derive(Component)]
pub struct SpriteAnimation {
    pub frames: Vec<Handle<Image>>,
    pub frame_duration: f32,
    pub timer: f32,
    pub current: usize,
}

pub fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut SpriteAnimation, &mut Sprite)>,
) {
    for (mut anim, mut sprite) in query.iter_mut() {
        anim.timer += time.delta_secs();
        if anim.timer < anim.frame_duration {
            continue;
        }
        anim.timer -= anim.frame_duration;
        anim.current = (anim.current + 1) % anim.frames.len();
        sprite.image = anim.frames[anim.current].clone();
    }
}
