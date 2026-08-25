//! 结算画面（胜利 / 失败）。

use bevy::prelude::*;
use bevy::ui::ZIndex;

use crate::assets::GameAssets;
use crate::input::SelectedPlant;
use crate::lawn::LawnOccupancy;
use crate::level::LevelRuntime;
use crate::state::{GameState, GameplayEntity};
use crate::ui::menebar::SunBank;
use crate::ui::plant_cards::PlantCards;

use super::despawn_recursive;

// ===== 组件 =====

#[derive(Component)]
struct EndScreenRoot;

#[derive(Component)]
struct EndScreenButton;

// ===== 插件 =====

pub struct EndScreenPlugin;

impl Plugin for EndScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Defeat), setup_end_screen)
            .add_systems(OnEnter(GameState::Victory), setup_end_screen)
            .add_systems(
                Update,
                handle_end_screen
                    .run_if(in_state(GameState::Defeat).or_eager(in_state(GameState::Victory))),
            )
            .add_systems(OnExit(GameState::Defeat), despawn_end_screen)
            .add_systems(OnExit(GameState::Victory), despawn_end_screen);
    }
}

// ===== Systems =====

fn setup_end_screen(
    mut commands: Commands,
    state: Res<State<GameState>>,
    assets: Res<GameAssets>,
) {
    let (title, color) = match state.get() {
        GameState::Victory => ("胜利！", Color::srgb(0.95, 0.85, 0.3)),
        _ => ("游戏失败", Color::srgb(0.85, 0.2, 0.2)),
    };
    let font = assets.font.clone();
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(1066.0),
                height: Val::Px(600.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ZIndex(1000),
            EndScreenRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(64.0),
                    ..default()
                },
                TextColor(color),
            ));
            parent
                .spawn((
                    Button,
                    EndScreenButton,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(60.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.82, 0.78, 0.62)),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("重新开始"),
                        TextFont {
                            font: FontSource::Handle(font.clone()),
                            font_size: FontSize::Px(26.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.15, 0.12, 0.05)),
                    ));
                });
        });
}

#[allow(clippy::too_many_arguments)]
fn handle_end_screen(
    interaction: Query<(&Interaction, Entity), (Changed<Interaction>, With<EndScreenButton>)>,
    gameplay: Query<Entity, With<GameplayEntity>>,
    children: Query<&Children>,
    mut selected: ResMut<SelectedPlant>,
    mut sun: ResMut<SunBank>,
    mut cards: ResMut<PlantCards>,
    mut runtime: ResMut<LevelRuntime>,
    mut occupancy: ResMut<LawnOccupancy>,
    level: Res<crate::config::LevelDefinition>,
    mut next: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    for (inter, entity) in interaction.iter() {
        if *inter != Interaction::Pressed {
            continue;
        }
        let _ = entity;
        let entities: Vec<Entity> = gameplay.iter().collect();
        for e in entities {
            despawn_recursive(&mut commands, e, &children);
        }
        selected.kind = None;
        *sun = SunBank::default();
        *cards = PlantCards::default();
        *runtime = LevelRuntime::default();
        *occupancy = LawnOccupancy::from_level(&level);
        next.set(GameState::Playing);
    }
}

fn despawn_end_screen(
    mut commands: Commands,
    query: Query<Entity, With<EndScreenRoot>>,
    children: Query<&Children>,
) {
    for entity in query.iter() {
        despawn_recursive(&mut commands, entity, &children);
    }
}
