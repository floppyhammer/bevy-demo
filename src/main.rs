use crate::animated_sprite::AnimatedSpritePlugin;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget::Image;
use bevy::render::texture::DefaultImageSampler;
use bevy::window::WindowMode;
use bevy_inspector_egui::WorldInspectorPlugin;

mod animated_sprite;
mod camera3d;
mod model_viewer;
mod player;
mod player_controller;

use crate::camera3d::spawn_camera;
use crate::model_viewer::ModelViewerPlugin;
use crate::player::player_setup;
use crate::player_controller::PlayerControllerPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(ClearColor {
            0: Color::rgb(0.1, 0.1, 0.1),
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        monitor: MonitorSelection::Index(1),
                        position: WindowPosition::Centered,
                        title: "Bevy Demo".to_string(),
                        width: 1280.0,
                        height: 720.0,
                        mode: WindowMode::Windowed,
                        ..default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(WorldInspectorPlugin::new())
        // .add_plugin(ModelViewerPlugin)
        .add_startup_system(player_setup)
        .add_plugin(PlayerControllerPlugin)
        .add_plugin(AnimatedSpritePlugin)
        .run();
}
