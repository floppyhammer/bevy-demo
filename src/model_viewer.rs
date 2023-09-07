use bevy::prelude::*;

use bevy::render::mesh::skinning::SkinnedMesh;
use bevy::{
    asset::LoadState,
    core_pipeline::Skybox,
    render::{
        render_resource::{TextureViewDescriptor, TextureViewDimension},
        renderer::RenderDevice,
        texture::CompressedImageFormats,
    },
};

use bevy_xpbd_3d::{math::*, prelude::*};

use crate::camera3d;
use crate::camera3d::PanOrbitCamera;

pub struct ModelViewerPlugin;

impl Plugin for ModelViewerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
            .add_systems(Startup, setup_model_viewer)
            .add_systems(
                Update,
                (
                    animate_light_direction,
                    skybox_asset_loaded,
                    name_skinned_meshes,
                    update_position_of_root_joints,
                ),
            )
            .add_systems(Update, camera3d::pan_orbit_camera)
            // Physics engine.
            .add_plugins(PhysicsPlugins::default())
            .insert_resource(Gravity(Vector::NEG_Y * 9.81));
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
        Position(Vec3::Y * 8.0),
        AngularVelocity(Vec3::new(2.5, 3.4, 1.6)),
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

#[derive(Component)]
struct Controllable {
    original_local_transform: Transform,
}

fn name_skinned_meshes(
    mut commands: Commands,
    query: Query<(&SkinnedMesh, std::option::Option<&Name>)>,
    name_query: Query<&Name>,
    parent_query: Query<&Parent>,
    children_query: Query<&Children>,
    transform_query: Query<(&Transform, &GlobalTransform)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut setup: Local<bool>,
) {
    let no_mesh = query.iter().len() == 0;
    if no_mesh {
        return;
    }

    if !*setup {
        *setup = true;
    } else {
        return;
    }

    println!("Skinned mesh joints:");

    let mut skinned_meshes = vec![];
    let mut root_hair_joint = vec![];

    // Find all hair root joints.
    for (mesh, name) in &query {
        if let Some(name) = name {
            println!("Node with skinned mesh: {}", name.as_str());

            if name.as_str() == "Body.baked.0" {
                for (joint_index, current_joint) in mesh.joints.iter().enumerate() {
                    let joint_name = name_query.get(*current_joint).unwrap();
                    println!("Skinned mesh: {}", joint_name.as_str());

                    skinned_meshes.push(*current_joint);

                    if joint_name.contains("HairJoint") {
                        let joint_parent = parent_query.get(*current_joint).unwrap();

                        let parent_name = name_query.get(joint_parent.get()).unwrap();

                        if parent_name.as_str() == "J_Bip_C_Head" {
                            root_hair_joint.push(*current_joint);
                        }
                    }
                }

                break;
            }
        }
    }

    println!("Skinned meshes: {:?}", skinned_meshes);
    println!("Root hair joints: {:?}", root_hair_joint);

    let marker_radius = 0.01;

    let marker_mesh = PbrBundle {
        mesh: meshes.add(
            Mesh::try_from(shape::Icosphere {
                radius: marker_radius as f32,
                ..default()
            })
                .unwrap(),
        ),
        material: materials.add(StandardMaterial::from(Color::RED)),
        ..default()
    };

    for joint in root_hair_joint {
        let (xform, global_xform) = transform_query.get(joint).unwrap();
        let global_position = global_xform.translation();

        // Spawn a kinematic body in the root joint.
        commands.entity(joint).insert((
            RigidBody::Kinematic,
            Controllable {
                original_local_transform: *xform,
            },
            Collider::ball(0.01),
            Position(Vector::new(global_position.x, global_position.y, global_position.z)),
        ));

        {
            let joint_marker = commands
                .spawn((
                    marker_mesh.clone(),
                ))
                .id();

            commands.entity(joint).push_children(&[joint_marker]);
        }

        println!("{:?}", joint);
        spawn_joints_recursively(&mut commands, &children_query, &transform_query, joint, 1, &mut materials, &mut meshes);
    }
}

fn update_position_of_root_joints(
    mut query: Query<(Entity, &mut Position, &Controllable)>,
    parent_query: Query<&Parent>,
    transform_query: Query<(&Transform, &GlobalTransform)>,
) {
    for (entity, mut pos, control) in query.iter_mut() {
        let parent_entity = parent_query.get(entity).unwrap().get();
        let (_, parent_global_xform) = transform_query.get(parent_entity).unwrap();

        let new_global_xform = parent_global_xform.mul_transform(control.original_local_transform);
        let translation = new_global_xform.translation();

        // pos.x = translation.x;
        // pos.y = translation.y;
        // pos.z = translation.z;
    }
}

fn spawn_joints_recursively(
    commands: &mut Commands,
    children_query: &Query<&Children>,
    transform_query: &Query<(&Transform, &GlobalTransform)>,
    parent_joint: Entity,
    depth: usize,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    // Reaches the leaf. A leaf is only a position marker for its parent's tail.
    let res = children_query.get(parent_joint);
    if res.is_err() {
        return;
    }

    let marker_radius = 0.01;

    let marker_mesh = PbrBundle {
        mesh: meshes.add(
            Mesh::try_from(shape::Icosphere {
                radius: marker_radius as f32,
                ..default()
            })
                .unwrap(),
        ),
        material: materials.add(StandardMaterial::from(Color::rgb(0.2, 0.7, 0.9))),
        ..default()
    };

    let children = res.unwrap();
    for child in children {
        for _ in 0..depth {
            print!("-");
        }
        println!("{:?}", child);

        let (xform, global_xform) = transform_query.get(*child).unwrap();

        let (parent_xform, parent_global_xform) = transform_query.get(parent_joint).unwrap();

        let global_position = global_xform.translation();

        let global_joint_length =
            (parent_global_xform.translation() - global_xform.translation()).length();

        // Should be equal to global_joint_length.
        let joint_distance = xform.translation.length();

        assert!((global_joint_length - joint_distance) < 0.0001);

        let spring_length = (joint_distance - 0.02) * 0.2;

        // Spawn dynamic body for the child bone, connecting it to its parent with joints.
        commands.entity(*child).insert((
            RigidBody::Dynamic,
            Position(Vector::new(
                global_position.x,
                global_position.y,
                global_position.z,
            )),
            Collider::ball(0.01),
            MassPropertiesBundle::new_computed(&Collider::ball(0.4), 1.0),
        ));

        {
            let joint_marker = commands
                .spawn((
                    marker_mesh.clone(),
                ))
                .id();

            commands.entity(*child).push_children(&[joint_marker]);
        }

        commands.spawn(
            DistanceJoint::new(*child, parent_joint)
                .with_local_anchor_1(Vector::new(0.0, -0.01, 0.0))
                .with_local_anchor_2(Vector::new(0.0, 0.01, 0.0))
                .with_rest_length(spring_length)
                .with_limits(0.9 * spring_length, 1.1 * spring_length)
                .with_linear_velocity_damping(0.1)
                .with_angular_velocity_damping(1.0)
                .with_compliance(1.0 / 100.0),
        );

        spawn_joints_recursively(commands, children_query, transform_query, *child, depth + 1, materials, meshes);
    }
}

#[derive(Component)]
struct SpringConstraint {
    entity1: Entity,
    entity2: Entity,
    rest_position1: Vector,
    rest_position2: Vector,
    lagrange: Scalar,
    compliance: Scalar,
}

impl PositionConstraint for SpringConstraint {}

impl XpbdConstraint<2> for SpringConstraint {
    fn entities(&self) -> [Entity; 2] {
        [self.entity1, self.entity2]
    }
    fn clear_lagrange_multipliers(&mut self) {
        self.lagrange = 0.0;
    }
    fn solve(&mut self, bodies: [&mut RigidBodyQueryItem; 2], dt: Scalar) {
        // let [body1, body2] = bodies;
        //
        // // Local attachment points at the centers of the bodies for simplicity
        // let [r1, r2] = [Vector::ZERO, Vector::ZERO];
        //
        // // Compute the positional difference
        // let delta_x = body1.current_position() - body2.current_position();
        //
        // // The current separation distance
        // let length = delta_x.length();
        //
        // // The value of the constraint function. When this is zero, the constraint is satisfied,
        // // and the distance between the bodies is the rest length.
        // let c = length - self.rest_length;
        //
        // // Avoid division by zero and unnecessary computation
        // if length <= 0.0 || c == 0.0 {
        //     return;
        // }
        //
        // // Normalized delta_x
        // let n = delta_x / length;
        //
        // // Compute generalized inverse masses (method from PositionConstraint)
        // let w1 = self.compute_generalized_inverse_mass(body1, r1, n);
        // let w2 = self.compute_generalized_inverse_mass(body2, r2, n);
        // let w = [w1, w2];
        //
        // // Constraint gradients, i.e. how the bodies should be moved
        // // relative to each other in order to satisfy the constraint
        // let gradients = [n, -n];
        //
        // // Compute Lagrange multiplier update, essentially the signed magnitude of the correction
        // let delta_lagrange =
        //     self.compute_lagrange_update(self.lagrange, c, &gradients, &w, self.compliance, dt);
        // self.lagrange += delta_lagrange;
        //
        // // Apply positional correction (method from PositionConstraint)
        // self.apply_positional_correction(body1, body2, delta_lagrange, n, r1, r2);
    }
}
