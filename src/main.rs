use crate::animated_sprite::AnimatedSpritePlugin;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget::Image;
use bevy::render::texture::DefaultImageSampler;
use bevy::window::{ExitCondition, WindowMode, WindowResolution};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::slice::Windows;

mod animated_sprite;
mod camera3d;
mod debug_label;
mod model_viewer;
mod player;
mod player_controller;

use crate::camera3d::spawn_camera;
use crate::debug_label::DebugLabelPlugin;
use crate::model_viewer::ModelViewerPlugin;
use crate::player::player_setup;
use crate::player_controller::PlayerControllerPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor {
            0: Color::rgb(0.1, 0.1, 0.1),
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(640.0, 480.0),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(ModelViewerPlugin)
        // .add_startup_system(player_setup)
        .add_plugin(DebugLabelPlugin)
        // .add_plugin(PlayerControllerPlugin)
        // .add_plugin(AnimatedSpritePlugin)
        .run();
}
