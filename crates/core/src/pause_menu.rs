use bevy::prelude::*;
use bevy::ui::ZIndex;

use crate::assets::GameAssets;
use crate::components::menebar::SunBank;
use crate::components::plant_cards::PlantCards;
use crate::input::SelectedPlant;
use crate::lawn::LawnOccupancy;
use crate::level::LevelRuntime;
use crate::state::{GameState, GameplayEntity};

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component)]
struct ContinueButton;

#[derive(Component)]
struct RestartButton;

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            toggle_pause
                .run_if(in_state(GameState::Playing).or_eager(in_state(GameState::Paused))),
        )
        .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
        .add_systems(
            Update,
            (button_interaction, handle_buttons).run_if(in_state(GameState::Paused)),
        )
        .add_systems(OnExit(GameState::Paused), despawn_pause_menu);
    }
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    match state.get() {
        GameState::Playing => next.set(GameState::Paused),
        GameState::Paused => next.set(GameState::Playing),
        _ => {}
    }
}

fn setup_pause_menu(mut commands: Commands, assets: Res<GameAssets>) {
    let font = assets.font.clone();
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(800.0),
                height: Val::Px(600.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            ZIndex(1000),
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("已暂停"),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(32.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
            ));
            parent
                .spawn((
                    Button,
                    ContinueButton,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(44.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.6, 0.3)),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("继续 (Esc)"),
                        TextFont {
                            font: FontSource::Handle(font.clone()),
                            font_size: FontSize::Px(18.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                    ));
                });
            parent
                .spawn((
                    Button,
                    RestartButton,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(44.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.6, 0.4, 0.2)),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("重新开始"),
                        TextFont {
                            font: FontSource::Handle(font.clone()),
                            font_size: FontSize::Px(18.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                    ));
                });
        });
}

fn button_interaction(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match *interaction {
            Interaction::Hovered => BackgroundColor(Color::srgb(0.45, 0.75, 0.45)),
            _ => BackgroundColor(Color::srgb(0.3, 0.6, 0.3)),
        };
    }
}

fn handle_buttons(
    interaction: Query<
        (&Interaction, Entity),
        (Changed<Interaction>, With<Button>),
    >,
    continue_buttons: Query<Entity, With<ContinueButton>>,
    restart_buttons: Query<Entity, With<RestartButton>>,
    gameplay: Query<Entity, With<GameplayEntity>>,
    children: Query<&Children>,
    mut selected: ResMut<SelectedPlant>,
    mut sun: ResMut<SunBank>,
    mut cards: ResMut<PlantCards>,
    mut runtime: ResMut<LevelRuntime>,
    mut occupancy: ResMut<LawnOccupancy>,
    mut next: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    for (interaction, entity) in interaction.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if continue_buttons.get(entity).is_ok() {
            next.set(GameState::Playing);
        } else if restart_buttons.get(entity).is_ok() {
            let entities: Vec<Entity> = gameplay.iter().collect();
            for e in entities {
                despawn_recursive(&mut commands, e, &children);
            }
            selected.kind = None;
            *sun = SunBank::default();
            *cards = PlantCards::default();
            *runtime = LevelRuntime::default();
            *occupancy = LawnOccupancy::default();
            next.set(GameState::Playing);
        }
    }
}

fn despawn_pause_menu(
    mut commands: Commands,
    query: Query<Entity, With<PauseMenuRoot>>,
    children: Query<&Children>,
) {
    for entity in query.iter() {
        despawn_recursive(&mut commands, entity, &children);
    }
}

fn despawn_recursive(
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
) {
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            despawn_recursive(commands, child, children_query);
        }
    }
    commands.entity(entity).despawn();
}
