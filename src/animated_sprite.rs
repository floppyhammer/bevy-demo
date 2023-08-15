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
        app.add_systems(Update, animated_sprite_system);
    }
}
