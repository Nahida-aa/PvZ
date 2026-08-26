use bevy::prelude::*;
use crate::assets::GameAssets;
use crate::level::LevelDefinition;
use crate::settings::CardConfig;
use crate::state::GameState;
use super::components::*;
use super::ui::build_card_selection_ui;

/// 卡片选择状态
#[derive(Resource)]
pub struct CardSelectionState {
    pub selected_cards: Vec<String>,
    pub max_cards: usize,
}

impl Default for CardSelectionState {
    fn default() -> Self {
        Self {
            selected_cards: Vec::new(),
            max_cards: 10, // 默认最大选卡数
        }
    }
}

/// 初始化选卡界面
pub fn setup_card_selection(
    mut commands: Commands,
    assets: Res<GameAssets>,
    level: Res<LevelDefinition>,
    card_config: Res<CardConfig>,
) {
    info!("进入选卡界面");
    build_card_selection_ui(&mut commands, &assets, &level, &card_config);
    commands.insert_resource(CardSelectionState {
        selected_cards: Vec::new(),
        max_cards: level.max_choosed_card_num as usize,
    });
}

/// 处理卡片点击
pub fn handle_card_click(
    interaction: Query<(&Interaction, &CardEntity), Changed<Interaction>>,
    mut selection: ResMut<CardSelectionState>,
    state: Res<State<GameState>>,
) {
    if *state.get() != GameState::ChoosingCards {
        return;
    }

    for (interaction, card) in interaction.iter() {
        if *interaction == Interaction::Pressed {
            if selection.selected_cards.contains(&card.plant_kind) {
                // 取消选择
                selection.selected_cards.retain(|c| c != &card.plant_kind);
                info!("取消选择: {}", card.plant_kind);
            } else if selection.selected_cards.len() < selection.max_cards {
                // 选择卡片
                selection.selected_cards.push(card.plant_kind.clone());
                info!("选择: {} ({}/{})", card.plant_kind, selection.selected_cards.len(), selection.max_cards);
            } else {
                info!("已达到最大选卡数: {}", selection.max_cards);
            }
        }
    }
}

/// 处理开始游戏按钮
pub fn handle_start_game(
    interaction: Query<&Interaction, (Changed<Interaction>, With<StartGameButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in interaction.iter() {
        if *interaction == Interaction::Pressed {
            info!("开始游戏！");
            next_state.set(GameState::Playing);
        }
    }
}

/// 显示/隐藏选卡界面
pub fn update_card_selection_visibility(
    state: Res<State<GameState>>,
    card_config: Res<CardConfig>,
    mut candidate_panel: Query<&mut Node, With<CardCandidatePanel>>,
) {
    let is_choosing = *state.get() == GameState::ChoosingCards;
    let cc = &card_config;
    let panel_h = cc.candidate_offset_y + cc.candidate_rows as f32 * (cc.card_slot_height + cc.candidate_card_gap_y) + 20.0;

    if let Ok(mut node) = candidate_panel.single_mut() {
        node.top = if is_choosing {
            Val::Px(87.0)
        } else {
            Val::Px(87.0 + panel_h)
        };
    }
}
