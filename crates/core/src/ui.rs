use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::input::SelectedPlant;
use crate::plant::PlantKind;
use crate::state::GameState;

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
struct HudRoot;

#[derive(Component)]
struct SunCounter;

#[derive(Component)]
struct PlantCard {
    kind: PlantKind,
}

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SunBank>()
            .init_resource::<SelectedPlant>()
            .add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(
                Update,
                (update_sun_counter, handle_card_click)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_hud(mut commands: Commands, assets: Res<GameAssets>) {
    let font = assets.font.clone();
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Px(596.0),
                height: Val::Px(87.0),
                ..default()
            },
            ImageNode::new(assets.chooser_bg.clone()),
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
                BackgroundColor(Color::srgb(0.93, 0.92, 0.66)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(21.0),
                    bottom: Val::Px(24.0),
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

            for ((kind, card_image), &x) in cards.iter().zip(card_x_positions.iter()) {
                let cost = kind.cost();
                parent
                    .spawn((
                        Button,
                        PlantCard { kind: *kind },
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
                            Text::new(format!("{cost}")),
                            TextFont {
                                font: FontSource::Handle(font.clone()),
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.0, 0.0, 0.0)),
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(4.0),
                            right: Val::Px(18.0),
                            ..default()
                        },
                        ));
                    });
            }
        });
}

fn update_sun_counter(
    bank: Res<SunBank>,
    mut query: Query<&mut Text, With<SunCounter>>,
) {
    if !bank.is_changed() {
        return;
    }
    for mut text in query.iter_mut() {
        **text = format!("{}", bank.amount);
    }
}

fn handle_card_click(
    mut interaction_query: Query<(&Interaction, &PlantCard, &mut BackgroundColor), Changed<Interaction>>,
    mut selected: ResMut<SelectedPlant>,
    bank: Res<SunBank>,
) {
    for (interaction, card, mut bg) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if bank.amount >= card.kind.cost() {
                    selected.kind = Some(card.kind);
                    *bg = BackgroundColor(Color::srgb(0.4, 0.8, 0.4));
                }
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.2, 0.5, 0.2));
            }
            _ => {}
        }
    }
}
