use crate::animated_sprite::AnimatedSpriteTimer;
use crate::player_controller::PlayerController;
use bevy::prelude::*;

pub fn player_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // Spawn a 2d camera.
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.camera.priority = 1;
    commands.spawn(camera_bundle);

    spawn_player(&mut commands, asset_server, texture_atlases);
}

pub fn spawn_player(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture = asset_server.load("textures/adventurer-v1.5-Sheet.png");

    // Create atlas from texture.
    let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(50.0, 37.0), 7, 16, None, None);

    let texture_atlas = texture_atlases.add(texture_atlas);

    let anim_fps = 10.0;

    let timer = Timer::from_seconds(1.0 / anim_fps, TimerMode::Repeating);

    commands
        .spawn(SpriteSheetBundle {
            texture_atlas,
            transform: Transform::from_scale(Vec3::splat(2.0)),
            ..Default::default()
        })
        .insert(AnimatedSpriteTimer(timer))
        .insert(PlayerController)
        .insert(Name::new("Player"));
}
