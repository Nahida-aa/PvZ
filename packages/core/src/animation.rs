use bevy::prelude::*;

/// Animation track interpolation type, matching Godot's `Animation.InterpolationType`.
///
/// Determines how values are blended between keyframes during animation playback.
/// Stored as integer in `.tres` / JSONC files (`tracks/N/interp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Interpolation {
    /// `interp = 0` — No interpolation, snaps to nearest keyframe value.
    /// Godot: `INTERPOLATION_NEAREST`
    Nearest = 0,
    /// `interp = 1` — Linear interpolation between keyframes (default).
    /// Godot: `INTERPOLATION_LINEAR`
    Linear = 1,
    /// `interp = 2` — Cubic interpolation, smoother than linear but more expensive.
    /// Godot: `INTERPOLATION_CUBIC`
    Cubic = 2,
    /// `interp = 3` — Linear interpolation with shortest-path rotation.
    /// Godot: `INTERPOLATION_LINEAR_ANGLE`
    LinearAngle = 3,
    /// `interp = 4` — Cubic interpolation with shortest-path rotation.
    /// Godot: `INTERPOLATION_CUBIC_ANGLE`
    CubicAngle = 4,
}

impl Interpolation {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Nearest,
            1 => Self::Linear,
            2 => Self::Cubic,
            3 => Self::LinearAngle,
            4 => Self::CubicAngle,
            _ => {
                eprintln!("Unknown interpolation type {v}, defaulting to Linear");
                Self::Linear
            }
        }
    }
}

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
