use bevy::prelude::*;

#[derive(Component, Default, Clone)]
pub(crate) struct PauseMenuRoot;

#[derive(Component, Default, Clone)]
pub(crate) struct PauseButtonMarker;

#[derive(Component, Default, Clone)]
pub(crate) struct ContinueButton;

#[derive(Component, Default, Clone)]
pub(crate) struct RestartButton;

#[derive(Component, Default, Clone)]
pub(crate) struct MainMenuButton;

#[derive(Component, Default, Clone)]
pub(crate) struct AlmanacButton;

#[derive(Component, Default, Clone)]
pub(crate) struct OptionsButton;

/// 记录按钮未按下时的原始 `top`，用于按下时轻微位移反馈。
#[derive(Component, Clone, Copy)]
pub(crate) struct OriginalTop(pub f32);

impl Default for OriginalTop {
    fn default() -> Self {
        Self(0.0)
    }
}

/// 滑动条类型标记。
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SliderType {
    Music,
    Sound,
    Speed,
}

impl Default for SliderType {
    fn default() -> Self {
        Self::Music
    }
}

/// 倍速显示标签（"倍速 1.0 倍"）。
#[derive(Component, Default, Clone)]
pub(crate) struct SpeedLabel;
