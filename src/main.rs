use crate::animated_sprite::AnimatedSpritePlugin;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget::Image;
use bevy::render::texture::DefaultImageSampler;
use bevy::window::{ExitCondition, WindowMode, WindowResolution};
use bevy_editor_pls::editor_window::{EditorWindow, EditorWindowContext};
use bevy_editor_pls::{egui, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::slice::Windows;

mod animated_sprite;
mod camera3d;
mod debug_label;
mod model_viewer;
mod morph_targets;
mod player;
mod player_controller;

mod morph_viewer_plugin;
mod vrm_gltf;
pub use gltf::json as gltf_json;

use crate::camera3d::spawn_camera;
use crate::debug_label::DebugLabelPlugin;
use crate::model_viewer::ModelViewerPlugin;
use crate::morph_targets::VrmPlugin;
use crate::morph_viewer_plugin::MorphViewerPlugin;
use crate::player::player_setup;
use crate::player_controller::PlayerControllerPlugin;

fn main() {
    let mut a = Transform::from_translation(Vec3::new(10.0, 0.0, 0.0));
    a.look_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Z);
    let mat = a.compute_matrix();
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor {
            0: Color::rgb(0.1, 0.1, 0.1),
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(1280.0, 720.0),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        // .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(ModelViewerPlugin)
        // .add_systems(Startup, player_setup)
        .add_plugins(DebugLabelPlugin)
        // .add_plugins(PlayerControllerPlugin)
        // .add_plugins(AnimatedSpritePlugin)
        .add_plugins((VrmPlugin, MorphViewerPlugin))
        // .add_plugins(EditorPlugin::default())
        .run();
}
