mod camera;
mod ui;

pub use camera::CameraPlugin;
pub use ui::HudPlugin;

use bevy::prelude::*;

/// Estado global da simulação: rodando ou pausada.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationState {
    #[default]
    Running,
    Paused,
}

/// Configuração do espaço (arena) onde os átomos se movem.
#[derive(Resource)]
pub struct Arena {
    pub half_width: f32,
    pub half_height: f32,
}

impl Default for Arena {
    fn default() -> Self {
        Self {
            half_width: 580.0,
            half_height: 320.0,
        }
    }
}
