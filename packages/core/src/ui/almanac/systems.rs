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
    mut pages: Query<(&mut Visibility, Option<&AlmanacIndexPage>, Option<&AlmanacPlantPage>)>,
) {
    for interaction in interaction.iter() {
        if *interaction == Interaction::Pressed {
            for (mut vis, index, plant) in pages.iter_mut() {
                if index.is_some() {
                    *vis = Visibility::Hidden;
                }
                if plant.is_some() {
                    *vis = Visibility::Inherited;
                }
            }
        }
    }
}

pub fn handle_return_button(
    interaction: Query<&Interaction, (Changed<Interaction>, With<AlmanacReturnButton>)>,
    mut pages: Query<(&mut Visibility, Option<&AlmanacIndexPage>, Option<&AlmanacPlantPage>)>,
) {
    for interaction in interaction.iter() {
        if *interaction == Interaction::Pressed {
            for (mut vis, index, plant) in pages.iter_mut() {
                if index.is_some() {
                    *vis = Visibility::Inherited;
                }
                if plant.is_some() {
                    *vis = Visibility::Hidden;
                }
            }
        }
    }
}

/// B0001 fix: one mutable access per component type, Without filters for disjoint queries.
pub(crate) fn handle_plant_card_click(
    interaction_query: Query<(&Interaction, &AlmanacPlantCard), Changed<Interaction>>,
    almanac: Res<AlmanacData>,
    asset_server: Res<AssetServer>,
    assets: Res<GameAssets>,
    mut images: Query<(
        &mut ImageNode,
        Option<&AlmanacDetailBg>,
        Option<&AlmanacDetailPreview>,
    ), Without<AlmanacDetailText>>,
    mut all_texts: Query<(
        &mut Text,
        Option<&AlmanacNameText>,
        Option<&AlmanacDescText>,
        Option<&AlmanacParamsText>,
        Option<&AlmanacHintText>,
        Option<&AlmanacIntroText>,
        Option<&AlmanacCostText>,
        Option<&AlmanacCooltimeText>,
    ), With<AlmanacDetailText>>,
) {
    for (interaction, card) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some((_name, data)) = almanac.plants.iter().find(|(n, _)| n == &card.card_name) else {
            continue;
        };

        for (mut img, bg, preview) in images.iter_mut() {
            if bg.is_some() {
                img.image = get_ground_bg(&data.bg, &assets);
            }
            if preview.is_some() {
                img.image = asset_server.load(&data.preview_image);
            }
        }

        for (mut text, name, desc, params, hint, intro, cost, cooltime) in all_texts.iter_mut() {
            if name.is_some() {
                **text = data.name.clone();
            } else if desc.is_some() {
                **text = data.desc.clone();
            } else if params.is_some() {
                let mut s = String::new();
                for (k, v) in &data.params {
                    s.push_str(&format!("{k}: {v}\n"));
                }
                **text = s;
            } else if hint.is_some() {
                **text = data.hint.clone().unwrap_or_default();
            } else if intro.is_some() {
                **text = data.intro.clone();
            } else if cost.is_some() {
                let c = card_visual::plant_sun_cost(&card.card_name);
                **text = format!("\u{82b1}\u{8d39}: {c}");
            } else if cooltime.is_some() {
                **text = "\u{51b7}\u{5374}\u{65f6}\u{95f4}: (\u{79d2})".to_string();
            }
        }

        info!("\u{56fe}\u{9274}\u{9009}\u{4e2d}: {}", data.name);
    }
}
