mod common;
mod eras;
mod physics;

use bevy::prelude::*;
use common::{CameraPlugin, SimulationState};
use eras::democritus::DemocritusPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "atom-sim — História do Modelo Atômico".into(),
                resolution: (1280u32, 720u32).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<SimulationState>()
        .add_plugins(CameraPlugin)
        .add_plugins(DemocritusPlugin)
        .run();
}
