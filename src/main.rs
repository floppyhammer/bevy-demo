use crate::animated_sprite::AnimatedSpritePlugin;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget::Image;
use bevy::render::texture::DefaultImageSampler;
use bevy::window::{ExitCondition, WindowMode, WindowResolution};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::slice::Windows;

mod animated_sprite;
mod camera;
mod debug_label;
mod morph_targets;
mod morph_viewer_plugin;
mod scene_viewer;
mod vrm_gltf;

pub use gltf::json as gltf_json;

use crate::debug_label::DebugLabelPlugin;
use crate::morph_targets::VrmPlugin;
use crate::morph_viewer_plugin::MorphViewerPlugin;
use crate::scene_viewer::SceneViewerPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
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
                .set(ImagePlugin::default_linear()),
        )
        // .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(SceneViewerPlugin)
        .add_plugins(DebugLabelPlugin)
        .add_plugins(AnimatedSpritePlugin)
        .add_plugins((VrmPlugin, MorphViewerPlugin))
        .run();
}
