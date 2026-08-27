use bevy::prelude::*;
use crate::assets::GameAssets;
use crate::level::LevelDefinition;
use crate::settings::CardConfig;
use crate::state::GameState;
use crate::ui::almanac;
use crate::ui::almanac::data::AlmanacData;
use crate::ui::seed_bank::SeedBankRoot;
use super::components::*;
use super::ui::build_card_selection_ui;
use crate::ui::menu::despawn_recursive;

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
    mut candidate_panel: Query<&mut Node, With<CardCandidatePanel>>,
) {
    let is_choosing = *state.get() == GameState::ChoosingCards;

    if let Ok(mut node) = candidate_panel.single_mut() {
        node.top = if is_choosing {
            Val::Px(87.0)
        } else {
            Val::Px(87.0 + 513.0) // SeedChooser_Background.png 高度
        };
    }
}

/// 图鉴模式下隐藏 seed_bank
pub fn update_seed_bank_visibility(
    state: Res<State<GameState>>,
    mut query: Query<&mut Visibility, With<SeedBankRoot>>,
) {
    let is_encyclopedia = *state.get() == GameState::Encyclopedia;
    for mut vis in query.iter_mut() {
        if is_encyclopedia {
            *vis = Visibility::Hidden;
        } else {
            *vis = Visibility::Inherited;
        }
    }
}

/// 处理查看图鉴按钮
pub fn handle_encyclopedia_button(
    interaction: Query<&Interaction, (Changed<Interaction>, With<EncyclopediaButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in interaction.iter() {
        if *interaction == Interaction::Pressed {
            info!("打开图鉴");
            next_state.set(GameState::Encyclopedia);
        }
    }
}

/// 进入图鉴状态时构建图鉴 UI
pub fn setup_encyclopedia(
    mut commands: Commands,
    assets: Res<GameAssets>,
    almanac: Res<AlmanacData>,
) {
    info!("进入图鉴");
    almanac::ui::build_encyclopedia(&mut commands, &assets, &almanac);
}

/// 离开图鉴状态时销毁图鉴 UI
pub fn cleanup_encyclopedia(
    mut commands: Commands,
    query: Query<Entity, With<almanac::components::EncyclopediaRoot>>,
    children_query: Query<&Children>,
) {
    for entity in query.iter() {
        despawn_recursive(&mut commands, entity, &children_query);
    }
}
