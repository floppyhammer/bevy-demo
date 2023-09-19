use crate::morph_viewer_plugin::WeightsControl;
use bevy::gltf::GltfExtras;
use bevy::prelude::*;
use bevy::render::mesh::skinning::SkinnedMesh;
use bevy_xpbd_3d::{math::*, prelude::*, SubstepSchedule, SubstepSet};
use gltf::Gltf;
use kira::{
    manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use rlip_sync::lip_sync::*;
use serde;
use std::cmp::{max, min};
use std::f32::consts::PI;
use std::time::SystemTime;

pub struct VrmPlugin;

/// In Hz.
const LIP_SYNC_SAMPLE_RATE: u32 = 20;

/// In ms.
const LIP_SYNC_SAMPLE_RANGE_LENGTH: u32 = 100;

impl Plugin for VrmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 1.0,
            ..default()
        })
        .add_systems(Startup, (setup))
        .add_systems(
            Update,
            (
                name_morphs,
                setup_animations,
                setup_spring_bones,
                update_shape,
                blow_wind,
            ),
        )
        // Physics engine.
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vector::NEG_Y * 9.8));

        // Get physics substep schedule and add our custom distance constraint
        let substeps = app
            .get_schedule_mut(SubstepSchedule)
            .expect("Add SubstepSchedule first");
        substeps.add_systems(
            solve_constraint::<SpringConstraint, 2>.in_set(SubstepSet::SolveUserConstraints),
        );
    }
}

#[derive(Resource)]
struct MorphData {
    anim: Handle<AnimationClip>,
    mesh: Handle<Mesh>,
}

#[derive(Resource)]
struct SpeechAudio {
    pub sound_data: StaticSoundData,
    pub lip_sync: LipSync,
    pub start_time: SystemTime,
    pub last_time_handled: u128,
    pub playing: bool,
}

#[derive(Component)]
struct MyAudio;

#[derive(Component)]
struct VrmData {
    pub spring_bone_roots: Vec<String>,
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    // Create an audio manager, which plays sounds and manages resources.
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/sound.ogg"),
            ..default()
        },
        MyAudio,
    ));

    let sound_data =
        match StaticSoundData::from_file("assets/sounds/sound.ogg", StaticSoundSettings::default())
        {
            Ok(data) => {
                println!("Loaded audio file.");
                data
            }
            Err(error) => {
                println!("Error loading audio file: {:?}", error);
                panic!();
            }
        };

    let mut lip_sync = LipSync::new();

    commands.insert_resource(SpeechAudio {
        sound_data,
        lip_sync,
        start_time: SystemTime::now(),
        last_time_handled: 0,
        playing: false,
    });

    commands.insert_resource(MorphData {
        anim: asset_server.load("models/AvatarSample_A.glb#Animation0"),
        mesh: asset_server.load("models/AvatarSample_A.glb#Mesh1/Primitive1"),
    });

    let mut vrm_scene = commands.spawn(SceneBundle {
        scene: asset_server.load("models/AvatarSample_A.glb#Scene0"),
        ..default()
    });

    {
        let gltf: gltf::Gltf<crate::vrm_gltf::GltfExtensions> =
            Gltf::open("assets/models/AvatarSample_A.glb").unwrap();
        // println!("{:?}", gltf.document.into_json());
        let json = gltf.document.as_json();

        let mut nodes = vec![];
        for node in gltf.document.nodes() {
            nodes.push((node.index(), node.name().unwrap()));
        }

        // println!("{:?}", json.to_string().unwrap());

        let extensions = json.extensions.as_ref().unwrap();

        let vrm = extensions.custom.vrm.as_ref().unwrap();

        let mut roots = vec![];
        for bone_group in &vrm.secondary_animation.bone_groups {
            for bone_index in &bone_group.bones {
                let bone = nodes[*bone_index as usize];
                println!("{:?} {:?}", bone.0, bone.1);
                roots.push(bone.1.to_string());
            }
        }

        vrm_scene.insert(VrmData {
            spring_bone_roots: roots,
        });

        let a = 1;
    }
}

/// Plays an [`AnimationClip`] from the loaded [`Gltf`] on the [`AnimationPlayer`] created by the spawned scene.
fn setup_animations(mut has_setup: Local<bool>, mut players: Query<(&Name, &mut AnimationPlayer)>) {
    if *has_setup {
        return;
    }
    for (name, mut player) in &mut players {
        // The name of the entity in the GLTF scene containing the AnimationPlayer for our morph targets is "Armature".
        if name.as_str() != "Armature" {
            continue;
        }
        // player.play(morph_data.anim.clone()).repeat();
        *has_setup = true;
    }

    // let res = morph_data.manager.play(morph_data.sound_data.clone());
    // if res.is_err() {
    //     println!("Playing sound failed!");
    // }
}

fn update_shape(
    controls: Option<ResMut<WeightsControl>>,
    morph_data: Res<MorphData>,
    mut speech_audio: ResMut<SpeechAudio>,
    mut morphs: Query<&mut MorphWeights>,
    music_controller: Query<&AudioSink, With<MyAudio>>,
    time: Res<Time>,
) {
    // if let Ok(sink) = music_controller.get_single() {
    //     sink.set_speed(((time.elapsed_seconds() / 5.0).sin() + 1.0).max(0.1));
    // }

    if !speech_audio.playing {
        return;
    }

    match speech_audio.start_time.elapsed() {
        Ok(elapsed) => {
            let current_time = elapsed.as_millis();

            let sample_interval_in_ms = (1000.0 / LIP_SYNC_SAMPLE_RATE as f32) as u128;

            if current_time > (speech_audio.last_time_handled + sample_interval_in_ms) {
                // println!("Time: {:?} {:?} {:?}", current_time, sample_interval_in_ms, speech_audio.last_time_handled);

                let current_time_in_sec = current_time as f32 / 1000.0;
                let half_sample_range_length_in_sec =
                    LIP_SYNC_SAMPLE_RANGE_LENGTH as f32 / 1000.0 * 0.5;

                // The range to sample.
                let frame_range = (
                    max(
                        ((current_time_in_sec - half_sample_range_length_in_sec)
                            * speech_audio.sound_data.sample_rate as f32)
                            as usize,
                        0,
                    ),
                    min(
                        ((current_time_in_sec + half_sample_range_length_in_sec)
                            * speech_audio.sound_data.sample_rate as f32)
                            as usize,
                        speech_audio.sound_data.frames.len(),
                    ),
                );

                let mut stream = Vec::new();

                for frame_index in frame_range.0..frame_range.1 {
                    stream.push(speech_audio.sound_data.frames[frame_index].left);
                }

                speech_audio.lip_sync.update(stream);
                let res = speech_audio.lip_sync.poll();
                if let Some(estimate) = res {
                    // println!("{:?}", estimate);

                    let Some(mut controls) = controls else {
                        return;
                    };
                    for (i, target) in controls.weights.iter_mut().enumerate() {
                        let new_weight = if estimate.vowel == i as i32 {
                            estimate.amount
                        } else {
                            0.0
                        };

                        target.weight = new_weight;
                    }
                }

                speech_audio.last_time_handled = current_time;
            }
        }
        Err(e) => {
            println!("Error: {e:?}");
        }
    }
}

fn pause(keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {}
}

/// You can get the target names in their corresponding [`Mesh`].
/// They are in the order of the weights.
fn name_morphs(
    mut has_printed: Local<bool>,
    morph_data: Res<MorphData>,
    meshes: Res<Assets<Mesh>>,
    music_controller: Query<&AudioSink, With<MyAudio>>,
    mut speech_audio: ResMut<SpeechAudio>,
) {
    if *has_printed {
        return;
    }

    let Some(mesh) = meshes.get(&morph_data.mesh) else {
        if let Ok(sink) = music_controller.get_single() {
            sink.pause();
        }
        return;
    };

    let Some(names) = mesh.morph_target_names() else {
        return;
    };

    println!("Shape Keys:");
    for name in names {
        println!("  {name}");
    }

    speech_audio.start_time = SystemTime::now();
    speech_audio.playing = true;
    if let Ok(sink) = music_controller.get_single() {
        sink.play();
    }

    *has_printed = true;
}

#[derive(Component)]
struct Controllable {
    original_local_transform: Transform,
}

fn setup_spring_bones(
    mut commands: Commands,
    skinned_mesh_query: Query<(Entity, &SkinnedMesh)>,
    name_query: Query<&Name>,
    parent_query: Query<&Parent>,
    children_query: Query<&Children>,
    transform_query: Query<(&Transform, &GlobalTransform)>,
    gltf_query: Query<&GltfExtras>,
    vrm_query: Query<&VrmData>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut setup: Local<bool>,
) {
    let no_skinned_mesh = skinned_mesh_query.iter().len() == 0;
    if no_skinned_mesh {
        return;
    }

    if !*setup {
        *setup = true;
    } else {
        return;
    }

    // for e in gltf_query.iter() {
    //     let json_string = serde_json::to_string_pretty(&e.value).unwrap();
    //     println!("{:?}", json_string);
    // }

    println!("Skinned mesh joints with spring bone:");

    let mut root_spring_bone_joints: Vec<Entity> = vec![];

    let mut found_scene_entity = false;

    for (entity, mesh) in &skinned_mesh_query {
        if found_scene_entity {
            break;
        }

        for joint in &mesh.joints {
            let joint_name = name_query.get(*joint).unwrap();

            // println!("{}", joint_name.as_str());

            for vrm in &vrm_query {
                if vrm
                    .spring_bone_roots
                    .contains(&joint_name.as_str().to_string())
                {
                    println!("{}", joint_name.as_str());
                    root_spring_bone_joints.push(*joint);
                    found_scene_entity = true;
                }
            }
        }
    }

    println!("Root spring bone joints: {:?}", root_spring_bone_joints);

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

    for joint in root_spring_bone_joints {
        let (xform, global_xform) = transform_query.get(joint).unwrap();
        let global_position = global_xform.translation().into();

        // Spawn a kinematic body in the root joint.
        commands.entity(joint).insert((
            RigidBody::Kinematic,
            Controllable {
                original_local_transform: *xform,
            },
            Collider::ball(0.01),
            Position(global_position),
        ));

        {
            let joint_marker = commands.spawn((marker_mesh.clone(),)).id();

            commands.entity(joint).push_children(&[joint_marker]);
        }

        spawn_joints_recursively(
            &mut commands,
            &children_query,
            &transform_query,
            joint,
            1,
            &mut materials,
            &mut meshes,
        );
    }

    println!("Spring bone setup finished");
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

        let global_position: Vector = global_xform.translation().into();

        let parent_global_position: Vector = parent_global_xform.translation().into();

        let global_joint_length =
            (parent_global_xform.translation() - global_xform.translation()).length();

        // Should be equal to global_joint_length.
        let joint_distance = xform.translation.length();

        assert!((global_joint_length - joint_distance) < 0.0001);

        let spring_length = (joint_distance - 0.02) * 0.2;

        // Spawn dynamic body for the child bone, connecting it to its parent with joints.
        commands.entity(*child).insert((
            RigidBody::Dynamic,
            Position(global_position),
            Collider::ball(0.01),
            MassPropertiesBundle::new_computed(&Collider::ball(0.01), 1.0),
        ));

        {
            let joint_marker = commands.spawn((marker_mesh.clone(),)).id();

            commands.entity(*child).push_children(&[joint_marker]);
        }

        let mut joint = FixedJoint::new(*child, parent_joint)
            .with_local_anchor_1(Vector::new(0.0, -0.01, 0.0))
            .with_local_anchor_2(Vector::new(0.0, 0.01, 0.0));
        joint.compliance = 0.01;

        // let joint2 = DistanceJoint::new(*child, parent_joint)
        //     .with_local_anchor_1(Vector::new(0.0, -0.01, 0.0))
        //     .with_local_anchor_2(Vector::new(0.0, 0.01, 0.0))
        //     .with_rest_length(spring_length.into())
        //     .with_limits(0.9 * spring_length, 1.1 * spring_length)
        //     .with_linear_velocity_damping(0.1)
        //     .with_angular_velocity_damping(1.0)
        //     .with_compliance(1.0 / 100.0);

        let joint3 = SpringConstraint {
            entity1: parent_joint,
            entity2: *child,
            relative_rest_position: parent_global_position - global_position,
            lagrange: 0.0,
            compliance: 1.0 / 100.0,
        };

        commands.spawn(joint3);

        // Apply wind force.
        let mut force = ExternalForce::default();
        force.apply_force(Vector::X);
        commands
            .entity(*child)
            .insert((RigidBody::Dynamic, Wind, force));

        spawn_joints_recursively(
            commands,
            children_query,
            transform_query,
            *child,
            depth + 1,
            materials,
            meshes,
        );
    }
}

#[derive(Component)]
struct SpringConstraint {
    entity1: Entity,
    entity2: Entity,
    // Relative position from entity2 to entity1.
    relative_rest_position: Vector,
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
        let [body1, body2] = bodies;

        // Local attachment points at the centers of the bodies for simplicity.
        let [r1, r2] = [Vector::ZERO, Vector::ZERO];

        // Compute the positional difference.
        let delta_pos = body1.current_position() - body2.current_position();

        // The current separation distance.
        let length = delta_pos.length();

        // The value of the constraint function. When this is zero, the constraint is satisfied.
        let c = delta_pos - self.relative_rest_position;

        // Avoid division by zero and unnecessary computation.
        if length <= 0.0 || c.length() == 0.0 {
            return;
        }

        let n = c.normalize();

        // Compute generalized inverse masses (method from PositionConstraint).
        let w1 = self.compute_generalized_inverse_mass(body1, r1, n);
        let w2 = self.compute_generalized_inverse_mass(body2, r2, n);
        let w = [w1, w2];

        // Constraint gradients, i.e. how the bodies should be moved
        // relative to each other in order to satisfy the constraint.
        let gradients = [n, -n];

        // Compute Lagrange multiplier update, essentially the signed magnitude of the correction.
        let delta_lagrange = self.compute_lagrange_update(
            self.lagrange,
            c.length(),
            &gradients,
            &w,
            self.compliance,
            dt,
        );
        self.lagrange += delta_lagrange;

        // Apply positional correction (method from PositionConstraint)
        self.apply_positional_correction(body1, body2, delta_lagrange, n, r1, r2);
    }
}

#[derive(Component)]
struct Wind;

fn blow_wind(time: Res<Time>, mut query: Query<(&mut ExternalForce, With<Wind>)>) {
    let f = Vector::X * 0.02 * (time.elapsed_seconds().sin() as f64 + 1.0);
    // println!("F: {}", f);

    for (mut force, _) in query.iter_mut() {
        force.clear();
        force.apply_force(f);
    }
}
