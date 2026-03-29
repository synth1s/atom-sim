mod camera;
pub mod ui;
pub mod tooltip;
pub mod export;
pub mod timeline;
pub mod transition;

pub use camera::CameraPlugin;
pub use ui::HudPlugin;
pub use tooltip::TooltipPlugin;
pub use export::ExportPlugin;
pub use timeline::TimelinePlugin;
pub use transition::TransitionPlugin;

use bevy::prelude::*;

/// Estado global da simulação: rodando ou pausada.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationState {
    #[default]
    Running,
    Paused,
}

/// Qual era histórica está ativa.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActiveEra {
    #[default]
    Democritus,
    Dalton,
    Thomson,
    Rutherford,
    Bohr,
    Sommerfeld,
    DeBroglie,
    Schrodinger,
    Dirac,
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
