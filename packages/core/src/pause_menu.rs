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
struct PauseButtonMarker;

#[derive(Component)]
struct ContinueButton;

#[derive(Component)]
struct RestartButton;

#[derive(Component)]
struct MainMenuButton;

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
            handle_buttons.run_if(in_state(GameState::Paused)),
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

fn spawn_small_button(
    parent: &mut ChildSpawnerCommands,
    marker: impl Bundle,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    label: &str,
    font: &Handle<Font>,
    bg: &Handle<Image>,
) {
    parent
        .spawn((
            Button,
            PauseButtonMarker,
            marker,
            ImageNode::new(bg.clone()),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                width: Val::Px(width),
                height: Val::Px(height),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::srgb(0.2, 0.15, 0.05)),
            ));
        });
}

fn setup_pause_menu(mut commands: Commands, assets: Res<GameAssets>) {
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
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            ZIndex(1000),
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Relative,
                        width: Val::Px(412.0),
                        height: Val::Px(483.0),
                        ..default()
                    },
                    ImageNode::new(assets.pause_menu_bg.clone()),
                ))
                .with_children(|panel| {
                    spawn_small_button(
                        panel,
                        RestartButton,
                        5.0,
                        167.0,
                        195.0,
                        37.0,
                        "重新开始",
                        &font,
                        &assets.small_button_bg,
                    );
                    spawn_small_button(
                        panel,
                        MainMenuButton,
                        5.0,
                        211.0,
                        195.0,
                        37.0,
                        "主菜单",
                        &font,
                        &assets.small_button_bg,
                    );
                    panel
                        .spawn((
                            Button,
                            PauseButtonMarker,
                            ContinueButton,
                            ImageNode::new(assets.pause_return_button.clone()),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(26.0),
                                top: Val::Px(383.0),
                                width: Val::Px(360.0),
                                height: Val::Px(100.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("返回游戏"),
                                TextFont {
                                    font: FontSource::Handle(font.clone()),
                                    font_size: FontSize::Px(28.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.1, 0.1, 0.1)),
                            ));
                        });
                });
        });
}

fn handle_buttons(
    interaction: Query<(&Interaction, Entity), (Changed<Interaction>, With<Button>)>,
    continue_buttons: Query<Entity, With<ContinueButton>>,
    restart_buttons: Query<Entity, With<RestartButton>>,
    mainmenu_buttons: Query<Entity, With<MainMenuButton>>,
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
        } else if restart_buttons.get(entity).is_ok()
            || mainmenu_buttons.get(entity).is_ok()
        {
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

// ===== 结算画面 (Victory / Defeat) =====

#[derive(Component)]
struct EndScreenRoot;

#[derive(Component)]
struct EndScreenButton;

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

fn handle_end_screen(
    interaction: Query<(&Interaction, Entity), (Changed<Interaction>, With<EndScreenButton>)>,
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
        *occupancy = LawnOccupancy::default();
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

