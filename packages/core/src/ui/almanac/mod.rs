pub mod components;
pub mod data;
pub mod systems;
pub mod ui;

use bevy::prelude::*;
use data::AlmanacData;
use systems::*;

use crate::assets::GameAssets;
use crate::state::GameState;
use ui::build_encyclopedia;

pub struct AlmanacPlugin;

impl Plugin for AlmanacPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AlmanacData::load());
        app.add_systems(OnEnter(GameState::Encyclopedia), setup_encyclopedia);
        app.add_systems(
            Update,
            (
                handle_encyclopedia_close,
                handle_encyclopedia_close_hover,
                handle_plant_button,
                handle_return_button,
                handle_plant_card_click,
            )
                .run_if(in_state(GameState::Encyclopedia)),
        );
    }
}

fn setup_encyclopedia(
    mut commands: Commands,
    assets: Res<GameAssets>,
    almanac: Res<AlmanacData>,
) {
    build_encyclopedia(&mut commands, &assets, &almanac);
}
