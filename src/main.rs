use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimatedTextPlugin)
        .add_plugin(AnimatedSpritePlugin)
        .run();
}

pub struct AnimatedTextPlugin;

impl Plugin for AnimatedTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_animated_text)
            .add_system(animated_text_system);
    }
}

fn setup_animated_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 2d camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section(
            "This text is in the 2D scene.",
            TextStyle {
                font: asset_server.load("fonts/VonwaonBitmap-12px.ttf"),
                font_size: 60.0,
                color: Color::WHITE,
            },
            TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            },
        ),
        ..Default::default()
    });
}

fn animated_text_system(time: Res<Time>, mut query: Query<&mut Transform, With<Text>>) {
    // `Transform.translation` will determine the location of the text.
    // `Transform.scale` and `Transform.rotation` do not yet affect text (though you can set the
    // size of the text via `Text.style.font_size`)
    for mut transform in query.iter_mut() {
        transform.translation.x = 100.0 * time.seconds_since_startup().sin() as f32;
        transform.translation.y = 100.0 * time.seconds_since_startup().cos() as f32;
    }
}

pub struct AnimatedSpritePlugin;

impl Plugin for AnimatedSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_animated_sprite)
            .add_system(animated_sprite_system);
    }
}

fn animated_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index as usize + 1) % texture_atlas.textures.len();
        }
    }
}

fn setup_animated_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("textures/adventurer-v1.5-Sheet.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(50.0, 37.0), 7, 16);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.1, true));
}
