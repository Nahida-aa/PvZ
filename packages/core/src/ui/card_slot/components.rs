use bevy::prelude::*;

/// 出战卡槽根节点
#[derive(Component)]
pub struct CardSlotRoot;

/// 待选卡面板根节点
#[derive(Component)]
pub struct CardCandidatePanel;

/// 卡片占位符（空卡位）
#[derive(Component)]
pub struct CardPlaceholder(pub usize);

/// 卡片实体标记
#[derive(Component)]
pub struct CardEntity {
    pub plant_kind: String,
    pub index: usize,
}

/// 已选卡片标记
#[derive(Component)]
pub struct SelectedCard;

/// 待选卡片标记
#[derive(Component)]
pub struct CandidateCard;

/// 开始游戏按钮
#[derive(Component)]
pub struct StartGameButton;

/// 阳光标签
#[derive(Component)]
pub struct SunLabel;
