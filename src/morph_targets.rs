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

use rlip_sync::lip_sync::*;
use std::time::SystemTime;

use kira::{
    manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use crate::morph_viewer_plugin::WeightsControl;

pub struct MorphTargetsPlugin;

impl Plugin for MorphTargetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 1.0,
            ..default()
        })
            .add_systems(Startup, setup)
            .add_systems(Update, (name_morphs, setup_animations, update_shape));
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
    pub last_sec_handled: u64,
}

#[derive(Component)]
struct MyAudio;


fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    // Create an audio manager, which plays sounds and manages resources.
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

    commands.spawn((AudioBundle {
        source: asset_server.load("sounds/sound.ogg"),
        ..default()
    },
                    MyAudio));

    let sound_data = match StaticSoundData::from_file("assets/sounds/sound.ogg", StaticSoundSettings::default()) {
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
        last_sec_handled: 0,
    });

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
) {
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

fn update_shape(controls: Option<ResMut<WeightsControl>>,
                morph_data: Res<MorphData>, mut speech_audio: ResMut<SpeechAudio>, mut morphs: Query<&mut MorphWeights>, music_controller: Query<&AudioSink, With<MyAudio>>, time: Res<Time>) {
    if let Ok(sink) = music_controller.get_single() {
        sink.set_speed(((time.elapsed_seconds() / 5.0).sin() + 1.0).max(0.1));
    }

    match speech_audio.start_time.elapsed() {
        Ok(elapsed) => {
            let current_sec = elapsed.as_secs();
            if current_sec > speech_audio.last_sec_handled {
                let frame_range = (
                    speech_audio.last_sec_handled * speech_audio.sound_data.sample_rate as u64,
                    current_sec * speech_audio.sound_data.sample_rate as u64,
                );

                let mut stream = Vec::new();

                for frame_index in frame_range.0..frame_range.1 {
                    if frame_index < speech_audio.sound_data.frames.len() as u64 {
                        stream.push(speech_audio.sound_data.frames[frame_index as usize].left);
                    }
                }

                speech_audio.lip_sync.update(stream);
                let res = speech_audio.lip_sync.poll();
                if let Some(estimate) = res {
                    println!("{:?}", estimate);

                    let Some(mut controls) = controls else { return; };
                    for (i, target) in controls.weights.iter_mut().enumerate() {
                        let Ok(mut weights) = morphs.get_mut(target.entity) else {
                            continue;
                        };

                        // To update individual morph target weights, get the `MorphWeights`
                        // component and call `weights_mut` to get access to the weights.
                        let weights_slice = weights.weights_mut();
                        let i = target.index;

                        let new_weight = if estimate.vowel == i as i32 { 1.0 } else { 0.0 };

                        weights_slice[i] = new_weight;
                        target.weight = new_weight;
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {e:?}");
        }
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
