use bevy::prelude::*;
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
use bevy::ui::ZIndex;

use crate::assets::GameAssets;
use crate::plant::PlantKind;
use crate::state::GameState;
use crate::components::plant_cards::PlantCards;

#[derive(Resource)]
pub struct SunBank {
    pub amount: u32,
}

impl Default for SunBank {
    fn default() -> Self {
        Self { amount: 150 }
    }
}

#[derive(Component)]
struct SunCounter;

#[derive(Component)]
pub struct PlantCard {
    pub kind: PlantKind,
    pub cooldown_timer: f32,
    pub cooldown_duration: f32,
}

#[derive(Component)]
pub struct CardCooldownOverlay;

#[derive(Component)]
pub struct CardSelectedOverlay;

impl Default for PlantCard {
    fn default() -> Self {
        Self {
            kind: PlantKind::Peashooter,
            cooldown_timer: 0.0,
            cooldown_duration: 0.0,
        }
    }
}

pub struct GameMenuBarPlugin;

impl Plugin for GameMenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SunBank>()
            .init_resource::<PlantCards>()
            .add_systems(OnEnter(GameState::Playing), setup_menubar)
            .add_systems(
                Update,
                (update_sun_counter, handle_card_click, sync_selected_overlay, cooldown_tick)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_menubar(mut commands: Commands, assets: Res<GameAssets>) {
    let font = assets.font.clone();
    commands
        .spawn((
            Node {
                width: Val::Px(596.0),
                height: Val::Px(87.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            ImageNode::new(assets.chooser_bg.clone()),
            ZIndex(-1),
        ))
        .with_children(|parent| {
            parent.spawn((
                SunCounter,
                Text::new("150"),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.15, 0.15, 0.4)),
                BackgroundColor(Color::NONE),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(35.0),
                    bottom: Val::Px(6.0),
                    width: Val::Px(35.0),
                    height: Val::Px(17.0),
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ));

            let card_x_positions: [f32; 2] = [77.0, 128.0];
            let cards: [(PlantKind, Handle<Image>); 2] = [
                (PlantKind::Peashooter, assets.card_peashooter.clone()),
                (PlantKind::Sunflower, assets.card_sunflower.clone()),
            ];
            let cooldowns: [f32; 2] = [7.5, 5.0];

            for (((kind, card_image), &x), &cooldown) in cards.iter().zip(card_x_positions.iter()).zip(cooldowns.iter()) {
                let cost = kind.cost();
                parent
                    .spawn((
                        Button,
                        PlantCard {
                            kind: *kind,
                            cooldown_timer: 0.0,
                            cooldown_duration: cooldown,
                        },
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(x),
                            top: Val::Px(8.0),
                            width: Val::Px(50.0),
                            height: Val::Px(70.0),
                            justify_content: JustifyContent::End,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            ImageNode::new(card_image.clone()),
                            Node {
                                width: Val::Px(50.0),
                                height: Val::Px(70.0),
                                ..default()
                            },
                        ));
                        parent.spawn((
                            CardCooldownOverlay,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                width: Val::Px(50.0),
                                height: Val::Px(70.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                        ));
                        parent.spawn((
                            CardSelectedOverlay,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                width: Val::Px(50.0),
                                height: Val::Px(70.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        ));
                        parent.spawn((
                            Text::new(format!("{cost}")),
                            TextFont {
                                font: FontSource::Handle(font.clone()),
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.0, 0.0, 0.0)),
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: Val::Px(2.0),
                                right: Val::Px(18.0),
                                ..default()
                            },
                        ));
                    });
            }
        });
}

fn update_sun_counter(bank: Res<SunBank>, mut query: Query<&mut Text, With<SunCounter>>) {
    if !bank.is_changed() {
        return;
    }
    for mut text in query.iter_mut() {
        **text = format!("{}", bank.amount);
    }
}

fn handle_card_click(
    _time: Res<Time>,
    mut selected: ResMut<crate::input::SelectedPlant>,
    bank: Res<SunBank>,
    cards: Res<PlantCards>,
    assets: Res<GameAssets>,
    mut commands: Commands,
    interaction_query: Query<(&Interaction, Entity, &PlantCard), Changed<Interaction>>,
) {
    for (interaction, _entity, card_data) in interaction_query.iter() {
        if let Interaction::Pressed = *interaction {
            let usable = bank.amount >= card_data.kind.cost() && cards.ready(&card_data.kind);
            if usable {
                selected.kind = Some(card_data.kind);
            } else {
                commands.spawn((
                    AudioPlayer::<AudioSource>(assets.cannot_choose_sound.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
        }
    }
}

fn sync_selected_overlay(
    selected: Res<crate::input::SelectedPlant>,
    cards: Query<(&PlantCard, &Children)>,
    mut overlay_query: Query<&mut BackgroundColor, With<CardSelectedOverlay>>,
) {
    for (card, children) in cards.iter() {
        let is_selected = selected.kind == Some(card.kind);
        for child in children.iter() {
            if let Ok(mut bg) = overlay_query.get_mut(child) {
                let alpha = if is_selected { 0.35 } else { 0.0 };
                bg.0 = Color::srgba(0.0, 0.0, 0.0, alpha);
            }
        }
    }
}

fn cooldown_tick(
    time: Res<Time>,
    mut cards: ResMut<PlantCards>,
    card_query: Query<(&PlantCard, &Children)>,
    mut overlay_query: Query<&mut BackgroundColor, With<CardCooldownOverlay>>,
    mut overlay_node_query: Query<&mut Node, With<CardCooldownOverlay>>,
) {
    cards.peashooter_remaining = (cards.peashooter_remaining - time.delta_secs()).max(0.0);
    cards.sunflower_remaining = (cards.sunflower_remaining - time.delta_secs()).max(0.0);

    for (card, children) in card_query.iter() {
        let remaining = cards.remaining(card.kind);
        let progress = if cards.ready(&card.kind) {
            0.0
        } else {
            (remaining / card.cooldown_duration).clamp(0.0, 1.0)
        };
        let overlay_height = progress * 70.0;
        for child in children.iter() {
            if let Ok(mut bg) = overlay_query.get_mut(child) {
                bg.0.set_alpha(0.6);
            }
            if let Ok(mut node) = overlay_node_query.get_mut(child) {
                node.height = Val::Px(overlay_height);
            }
        }
    }
}
