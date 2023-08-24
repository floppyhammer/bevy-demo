//! Controls morph targets in a loaded scene.
//!
//! Illustrates:
//!
//! - How to access and modify individual morph target weights.
//!   See the [`update_weights`] system for details.
//! - How to read morph target names in [`name_morphs`].
//! - How to play morph target animations in [`setup_animations`].

use bevy::prelude::*;
use std::f32::consts::PI;

pub struct MorphTargetsPlugin;

impl Plugin for MorphTargetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 1.0,
            ..default()
        })
            .add_systems(Startup, setup)
            .add_systems(Update, (name_morphs, setup_animations));
    }
}

#[derive(Resource)]
struct MorphData {
    anim: Handle<AnimationClip>,
    mesh: Handle<Mesh>,
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(MorphData {
        anim: asset_server.load("models/AvatarSample_A_With_Morph_Anim.glb#Animation0"),
        mesh: asset_server.load("models/AvatarSample_A_With_Morph_Anim.glb#Mesh1/Primitive1"),
    });
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/AvatarSample_A_With_Morph_Anim.glb#Scene0"),
        ..default()
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 19350.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_z(PI / 2.0)),
        ..default()
    });
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(3.0, 2.1, 10.2).looking_at(Vec3::ZERO, Vec3::Y),
    //     ..default()
    // });
}

/// Plays an [`AnimationClip`] from the loaded [`Gltf`] on the [`AnimationPlayer`] created by the spawned scene.
fn setup_animations(
    mut has_setup: Local<bool>,
    mut players: Query<(&Name, &mut AnimationPlayer)>,
    morph_data: Res<MorphData>,
) {
    if *has_setup {
        return;
    }
    for (name, mut player) in &mut players {
        // The name of the entity in the GLTF scene containing the AnimationPlayer for our morph targets is "Armature".
        if name.as_str() != "Armature" {
            continue;
        }
        player.play(morph_data.anim.clone()).repeat();
        *has_setup = true;
    }
}

/// You can get the target names in their corresponding [`Mesh`].
/// They are in the order of the weights.
fn name_morphs(
    mut has_printed: Local<bool>,
    morph_data: Res<MorphData>,
    meshes: Res<Assets<Mesh>>,
) {
    if *has_printed {
        return;
    }

    let Some(mesh) = meshes.get(&morph_data.mesh) else { return; };
    let Some(names) = mesh.morph_target_names() else { return; };
    for name in names {
        println!("  {name}");
    }
    *has_printed = true;
}
