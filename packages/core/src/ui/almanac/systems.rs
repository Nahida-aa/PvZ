use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::state::GameState;
use crate::ui::card_visual;

use super::components::*;
use super::data::{get_ground_bg, AlmanacData};

pub fn handle_encyclopedia_close(
    interaction: Query<&Interaction, (Changed<Interaction>, With<AlmanacCloseButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in interaction.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::ChoosingCards);
        }
    }
}

pub fn handle_encyclopedia_close_hover(
    interaction: Query<(&Interaction, &Children), (Changed<Interaction>, With<AlmanacCloseButton>)>,
    mut img_query: Query<&mut ImageNode, With<AlmanacCloseImage>>,
    assets: Res<GameAssets>,
) {
    for (interaction, children) in interaction.iter() {
        for child in children.iter() {
            if let Ok(mut img) = img_query.get_mut(child) {
                match *interaction {
                    Interaction::Hovered => {
                        img.image = assets.almanac_close_button_highlight.clone();
                    }
                    _ => {
                        img.image = assets.almanac_close_button.clone();
                    }
                }
            }
        }
    }
}

pub fn handle_plant_button(
    interaction: Query<&Interaction, (Changed<Interaction>, With<AlmanacPlantButton>)>,
    mut index_page: Query<&mut Visibility, With<AlmanacIndexPage>>,
    mut plant_page: Query<&mut Visibility, With<AlmanacPlantPage>>,
) {
    for interaction in interaction.iter() {
        if *interaction == Interaction::Pressed {
            for mut vis in index_page.iter_mut() {
                *vis = Visibility::Hidden;
            }
            for mut vis in plant_page.iter_mut() {
                *vis = Visibility::Inherited;
            }
        }
    }
}

pub fn handle_return_button(
    interaction: Query<&Interaction, (Changed<Interaction>, With<AlmanacReturnButton>)>,
    mut index_page: Query<&mut Visibility, With<AlmanacIndexPage>>,
    mut plant_page: Query<&mut Visibility, With<AlmanacPlantPage>>,
) {
    for interaction in interaction.iter() {
        if *interaction == Interaction::Pressed {
            for mut vis in index_page.iter_mut() {
                *vis = Visibility::Inherited;
            }
            for mut vis in plant_page.iter_mut() {
                *vis = Visibility::Hidden;
            }
        }
    }
}

/// B0001 fix: single query for all AlmanacDetailText entities, dispatch by specific marker.
pub fn handle_plant_card_click(
    interaction_query: Query<(&Interaction, &AlmanacPlantCard), Changed<Interaction>>,
    almanac: Res<AlmanacData>,
    asset_server: Res<AssetServer>,
    mut bg_query: Query<&mut ImageNode, With<AlmanacDetailBg>>,
    mut preview_query: Query<&mut ImageNode, (With<AlmanacDetailPreview>, Without<AlmanacDetailBg>)>,
    mut texts: ParamSet<(
        Query<(&mut Text, &AlmanacNameText), With<AlmanacDetailText>>,
        Query<(&mut Text, &AlmanacDescText), With<AlmanacDetailText>>,
        Query<(&mut Text, &AlmanacParamsText), With<AlmanacDetailText>>,
        Query<(&mut Text, &AlmanacHintText), With<AlmanacDetailText>>,
        Query<(&mut Text, &AlmanacIntroText), With<AlmanacDetailText>>,
        Query<(&mut Text, &AlmanacCostText), With<AlmanacDetailText>>,
        Query<(&mut Text, &AlmanacCooltimeText), With<AlmanacDetailText>>,
    )>,
    mut hint_node: Query<&mut Node, (With<AlmanacHintText>, Without<AlmanacDetailText>)>,
    assets: Res<GameAssets>,
) {
    for (interaction, card) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some((_name, data)) = almanac.plants.iter().find(|(n, _)| n == &card.card_name) else {
            continue;
        };

        for mut img in bg_query.iter_mut() {
            img.image = get_ground_bg(&data.bg, &assets);
        }
        for mut img in preview_query.iter_mut() {
            img.image = asset_server.load(&data.preview_image);
        }

        if let Ok((mut text, _)) = texts.p0().single_mut() {
            **text = data.name.clone();
        }
        if let Ok((mut text, _)) = texts.p1().single_mut() {
            **text = data.desc.clone();
        }
        if let Ok((mut text, _)) = texts.p2().single_mut() {
            let mut s = String::new();
            for (k, v) in &data.params {
                s.push_str(&format!("{k}: {v}\n"));
            }
            **text = s;
        }
        if let Ok((mut text, _)) = texts.p3().single_mut() {
            if let Some(ref hint) = data.hint {
                **text = hint.clone();
            }
        }
        if let Ok(mut node) = hint_node.single_mut() {
            node.display = if data.hint.is_some() { Display::Flex } else { Display::None };
        }
        if let Ok((mut text, _)) = texts.p4().single_mut() {
            **text = data.intro.clone();
        }

        let cost = card_visual::plant_sun_cost(&card.card_name);
        if let Ok((mut text, _)) = texts.p5().single_mut() {
            **text = format!("\u{82b1}\u{8d39}: {cost}");
        }
        if let Ok((mut text, _)) = texts.p6().single_mut() {
            **text = "\u{51b7}\u{5374}\u{65f6}\u{95f4}: (\u{79d2})".to_string();
        }

        info!("\u{56fe}\u{9274}\u{9009}\u{4e2d}: {}", data.name);
    }
}
