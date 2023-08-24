use crate::camera3d;
use crate::camera3d::spawn_camera;
use bevy::prelude::*;

pub struct ModelViewerPlugin;

impl Plugin for ModelViewerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
            .add_systems(Startup, model_viewer_setup)
            .add_systems(Update, animate_light_direction)
            .add_systems(Update, camera3d::pan_orbit_camera);
    }
}

fn model_viewer_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(SceneBundle {
            // Note that we have to include the `Scene0` label.
            scene: asset_server.load("models/AvatarSample_A.glb#Scene0"),
            ..default()
        })
        .insert(Name::new("Model"));

    // Add a camera.
    spawn_camera(&mut commands);

    const HALF_SIZE: f32 = 1.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight::default(),
        ..Default::default()
    });
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in query.iter_mut() {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * std::f32::consts::TAU / 10.0,
            -std::f32::consts::FRAC_PI_4,
        );
    }
}
