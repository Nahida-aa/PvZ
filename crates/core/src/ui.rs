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
                width: Val::Percent(100.0),
                height: Val::Px(87.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(0.0), Val::Px(0.0)),
                ..default()
            },
            ImageNode::new(assets.chooser_bg.clone()),
        ))
        .with_children(|parent| {
            parent.spawn((
                SunCounter,
                Text::new("☀ 150"),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.1)),
                Node {
                    margin: UiRect::right(Val::Px(20.0)),
                    ..default()
                },
            ));

            let cards: [(PlantKind, Handle<Image>, &str); 2] = [
                (PlantKind::Peashooter, assets.card_peashooter.clone(), "Peashooter"),
                (PlantKind::Sunflower, assets.card_sunflower.clone(), "Sunflower"),
            ];

            for (kind, card_image, _name) in cards {
                let cost = kind.cost();
                parent
                    .spawn((
                        Button,
                        PlantCard { kind },
                        Node {
                            width: Val::Px(70.0),
                            height: Val::Px(60.0),
                            margin: UiRect::horizontal(Val::Px(4.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            ImageNode::new(card_image),
                            Node {
                                width: Val::Px(50.0),
                                height: Val::Px(35.0),
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
                            TextColor(Color::srgb(1.0, 0.8, 0.3)),
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
        **text = format!("☀ {}", bank.amount);
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
