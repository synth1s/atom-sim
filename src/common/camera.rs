use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, camera_zoom);
    }
}

#[derive(Component)]
pub struct MainCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

fn camera_zoom(
    mut scroll_events: MessageReader<MouseWheel>,
    mut query: Query<&mut Projection, With<MainCamera>>,
) {
    for event in scroll_events.read() {
        for mut projection in query.iter_mut() {
            if let Projection::Orthographic(ref mut ortho) = *projection {
                let zoom_factor = 1.1;
                if event.y > 0.0 {
                    ortho.scale /= zoom_factor;
                } else if event.y < 0.0 {
                    ortho.scale *= zoom_factor;
                }
                ortho.scale = ortho.scale.clamp(0.2, 5.0);
            }
        }
    }
}
