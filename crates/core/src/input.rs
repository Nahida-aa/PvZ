use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::lawn::GridPos;
use crate::plant::{PlantKind, SpawnPlant};
use crate::schedule::GameSet;
use crate::state::GameState;
use crate::components::menebar::SunBank;
use crate::components::plant_cards::PlantCards;

#[derive(Resource, Default)]
pub struct SelectedPlant {
    pub kind: Option<PlantKind>,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedPlant>()
            .add_systems(
            Update,
            handle_click_to_place
                .in_set(GameSet::Spawn)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn handle_click_to_place(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut spawner: MessageWriter<SpawnPlant>,
    selected: Res<SelectedPlant>,
    mut bank: ResMut<SunBank>,
    mut cards: ResMut<PlantCards>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(plant) = selected.kind else {
        return;
    };
    if bank.amount < plant.cost() {
        return;
    }
    let Ok(window) = window.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        return;
    };
    let Some(grid_pos) = GridPos::from_world(world_pos) else {
        return;
    };
    bank.amount -= plant.cost();
    spawner.write(SpawnPlant { kind: plant, pos: grid_pos });
    cards.trigger(&plant);
}
