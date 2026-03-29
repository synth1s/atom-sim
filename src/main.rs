mod common;
mod eras;
mod physics;

use bevy::prelude::*;
use common::{ActiveEra, CameraPlugin, SimulationState};
use eras::democritus::DemocritusPlugin;
use eras::dalton::DaltonPlugin;
use eras::thomson::ThomsonPlugin;
use eras::rutherford::RutherfordPlugin;
use eras::bohr::BohrPlugin;
use eras::sommerfeld::SommerfeldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "atom-sim \u{2014} Historia do Modelo Atomico".into(),
                resolution: (1280u32, 720u32).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<SimulationState>()
        .init_state::<ActiveEra>()
        .add_plugins(CameraPlugin)
        .add_plugins(DemocritusPlugin)
        .add_plugins(DaltonPlugin)
        .add_plugins(ThomsonPlugin)
        .add_plugins(RutherfordPlugin)
        .add_plugins(BohrPlugin)
        .add_plugins(SommerfeldPlugin)
        .run();
}
