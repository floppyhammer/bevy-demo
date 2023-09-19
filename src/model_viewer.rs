use bevy::prelude::*;

use bevy::math::DVec3;
use bevy::{
    asset::LoadState,
    core_pipeline::Skybox,
    render::{
        render_resource::{TextureViewDescriptor, TextureViewDimension},
        renderer::RenderDevice,
        texture::CompressedImageFormats,
    },
};

use crate::camera3d;
use crate::camera3d::PanOrbitCamera;
use bevy_xpbd_3d::{math::*, prelude::*};

pub struct ModelViewerPlugin;

impl Plugin for ModelViewerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_systems(Startup, setup_model_viewer)
        .add_systems(Update, (animate_light_direction, skybox_asset_loaded))
        .add_systems(Update, (camera3d::pan_orbit_camera));
    }
}

#[derive(Resource)]
struct Cubemap {
    is_loaded: bool,
    index: usize,
    image_handle: Handle<Image>,
}

fn setup_model_viewer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(8.0))),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(8.0, 0.002, 8.0),
    ));

    // Cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            ..default()
        },
        RigidBody::Dynamic,
        Position(DVec3::Y * 8.0),
        AngularVelocity(DVec3::new(2.5, 3.4, 1.6)),
        Collider::cuboid(1.0, 1.0, 1.0),
    ));

    // Add skybox.
    let skybox_handle = asset_server.load("textures/Ryfjallet_cubemap.png");

    // Ambient light
    // NOTE: The ambient light is used to scale how bright the environment map is so with a bright
    // environment map, use an appropriate color and brightness to match
    // commands.insert_resource(AmbientLight {
    //     color: Color::rgb_u8(210, 220, 240),
    //     brightness: 1.0,
    // });

    // Add a gltf scene.
    commands
        .spawn(SceneBundle {
            // Note that we have to include the `Scene0` label.
            scene: asset_server.load("models/LivingRoom.glb#Scene0"),
            transform: Transform::from_scale(Vec3::new(0.01, 0.01, 0.01)),
            ..default()
        })
        .insert(Name::new("Model"));

    // Add a camera.
    {
        let translation = Vec3::new(-2.0, 4.0, -5.0);
        let radius = translation.length();

        commands
            .spawn((
                Camera3dBundle {
                    transform: Transform::from_translation(translation)
                        .looking_at(Vec3::ZERO, Vec3::Y),
                    ..Default::default()
                },
                // Add skybox as component to the camera.
                Skybox(skybox_handle.clone()),
            ))
            .insert(PanOrbitCamera {
                radius,
                ..Default::default()
            });
    }

    commands.insert_resource(Cubemap {
        is_loaded: false,
        index: 0,
        image_handle: skybox_handle,
    });

    // Add a light.
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight::default(),
        ..Default::default()
    });
}

fn skybox_asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if !cubemap.is_loaded
        && asset_server.get_load_state(cubemap.image_handle.clone_weak()) == LoadState::Loaded
    {
        let image = images.get_mut(&cubemap.image_handle).unwrap();
        // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
        // so they appear as one texture. The following code reconfigures the texture as necessary.
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array(
                image.texture_descriptor.size.height / image.texture_descriptor.size.width,
            );
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        for mut skybox in &mut skyboxes {
            skybox.0 = cubemap.image_handle.clone();
        }

        cubemap.is_loaded = true;
    }
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in query.iter_mut() {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * std::f32::consts::TAU / 100.0,
            -std::f32::consts::FRAC_PI_4,
        );
    }
}
