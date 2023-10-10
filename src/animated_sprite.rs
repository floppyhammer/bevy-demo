use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;

#[derive(Component, Deref, DerefMut)]
pub struct AnimatedSpriteTimer(pub Timer);

fn animated_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimatedSpriteTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        // Update timer.
        timer.tick(time.delta());

        if timer.finished() {
            // Get atlas.
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();

            // Update sprite frame.
            sprite.index = (sprite.index as usize + 1) % texture_atlas.textures.len();
        }
    }
}

pub struct AnimatedSpritePlugin;

impl Plugin for AnimatedSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_animated_sprite)
            .add_systems(Update, animated_sprite_system);
    }
}

pub fn spawn_animated_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // Spawn a 2d camera.
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.camera.order = 1;
    camera_bundle.camera_2d.clear_color = ClearColorConfig::None;
    commands.spawn(camera_bundle);

    let texture = asset_server.load("textures/qubodup-loading2-Sheet.png");

    const SPRITE_SIZE: f32 = 78.0;

    // Create atlas from texture.
    let texture_atlas = TextureAtlas::from_grid(texture, Vec2::splat(SPRITE_SIZE), 4, 1, None, None);

    let texture_atlas = texture_atlases.add(texture_atlas);

    let anim_fps = 2.0;

    let timer = Timer::from_seconds(1.0 / anim_fps, TimerMode::Repeating);

    commands
        .spawn(SpriteSheetBundle {
            texture_atlas,
            transform: Transform::from_translation(Vec3::new(200.0, 200.0, 0.0)) * Transform::from_scale(Vec3::splat(0.5)),
            ..Default::default()
        })
        .insert(AnimatedSpriteTimer(timer));
}
