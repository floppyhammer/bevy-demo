//! Controls morph targets in a loaded scene.
//!
//! Illustrates:
//!
//! - How to access and modify individual morph target weights.
//!   See the [`update_weights`] system for details.
//! - How to read morph target names in [`name_morphs`].
//! - How to play morph target animations in [`setup_animations`].

use std::cmp::{max, min};
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

/// In Hz.
const LIP_SYNC_SAMPLE_RATE: u32 = 20;

/// In ms.
const LIP_SYNC_SAMPLE_RANGE_LENGTH: u32 = 100;

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
    pub last_time_handled: u128,
    pub playing: bool,
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
        last_time_handled: 0,
        playing: false,
    });

    commands.insert_resource(MorphData {
        anim: asset_server.load("models/Animal.glb#Animation0"),
        mesh: asset_server.load("models/Animal.glb#Mesh0/Primitive0"),

    });

    commands.spawn(SceneBundle {
        scene: asset_server.load("models/Animal.glb#Scene0"),
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
    // if let Ok(sink) = music_controller.get_single() {
    //     sink.set_speed(((time.elapsed_seconds() / 5.0).sin() + 1.0).max(0.1));
    // }

    if !speech_audio.playing { return; }

    match speech_audio.start_time.elapsed() {
        Ok(elapsed) => {
            let current_time = elapsed.as_millis();

            let sample_interval_in_ms = (1000.0 / LIP_SYNC_SAMPLE_RATE as f32) as u128;

            if current_time > (speech_audio.last_time_handled + sample_interval_in_ms) {
                // println!("Time: {:?} {:?} {:?}", current_time, sample_interval_in_ms, speech_audio.last_time_handled);

                let current_time_in_sec = current_time as f32 / 1000.0;
                let half_sample_range_length_in_sec = LIP_SYNC_SAMPLE_RANGE_LENGTH as f32 / 1000.0 * 0.5;

                // The range to sample.
                let frame_range = (
                    max(((current_time_in_sec - half_sample_range_length_in_sec) * speech_audio.sound_data.sample_rate as f32) as usize, 0),
                    min(((current_time_in_sec + half_sample_range_length_in_sec) * speech_audio.sound_data.sample_rate as f32) as usize, speech_audio.sound_data.frames.len()),
                );

                let mut stream = Vec::new();

                for frame_index in frame_range.0..frame_range.1 {
                    stream.push(speech_audio.sound_data.frames[frame_index].left);
                }

                speech_audio.lip_sync.update(stream);
                let res = speech_audio.lip_sync.poll();
                if let Some(estimate) = res {
                    // println!("{:?}", estimate);

                    let Some(mut controls) = controls else { return; };
                    for (i, target) in controls.weights.iter_mut().enumerate() {
                        let new_weight = if estimate.vowel == i as i32 { estimate.amount } else { 0.0 };

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
    meshes: Res<Assets<Mesh>>, music_controller: Query<&AudioSink, With<MyAudio>>, mut speech_audio: ResMut<SpeechAudio>,
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

    let Some(names) = mesh.morph_target_names() else { return; };

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
