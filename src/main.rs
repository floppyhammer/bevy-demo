use bevy::prelude::*;
use bevy::window::WindowMode;

mod camera3d;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Bevy Demo".to_string(),
            width: 1280.0,
            height: 720.0,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .insert_resource(ClearColor { 0: Color::rgb(0.1, 0.1, 0.1) })
        .add_plugins(DefaultPlugins)
        //.add_plugin(ModelViewerPlugin)
        .add_plugin(AnimatedTextPlugin)
        .add_plugin(AnimatedSpritePlugin)
        .run();
}

pub struct AnimatedTextPlugin;

impl Plugin for AnimatedTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(animated_text_setup)
            .add_system(animated_text_system);
    }
}

fn animated_text_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 2d camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section(
            "Some Text",
            TextStyle {
                font: asset_server.load("fonts/VonwaonBitmap-12px.ttf"),
                font_size: 48.0,
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
        app.add_startup_system(animated_sprite_setup)
            .add_system(animated_sprite_system);
    }
}

fn animated_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        // Advance timer.
        timer.tick(time.delta());

        if timer.finished() {
            // Get atlas.
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();

            // Update sprite frame.
            sprite.index = (sprite.index as usize + 1) % texture_atlas.textures.len();
        }
    }
}

fn animated_sprite_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("textures/adventurer-v1.5-Sheet.png");

    // Create atlas from texture.
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(50.0, 37.0), 7, 16);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let sprite_fps = 10.0;

    let translation = Vec3::new(-2.0, 2.5, 5.0);
    let radius = translation.length();

    commands.spawn_bundle(SpriteSheetBundle {
        texture_atlas: texture_atlas_handle,
        transform: Transform::from_scale(Vec3::splat(2.0)),
        ..Default::default()
    }).insert(Timer::from_seconds(1.0 / sprite_fps, true));
}

pub struct ModelViewerPlugin;

impl Plugin for ModelViewerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
            .add_startup_system(model_viewer_setup)
            .add_system(animate_light_direction)
            .add_system(camera3d::pan_orbit_camera);
    }
}

fn model_viewer_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_scene(asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"));
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        ..Default::default()
    });
    const HALF_SIZE: f32 = 1.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..Default::default()
            },
            shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in query.iter_mut() {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.seconds_since_startup() as f32 * std::f32::consts::TAU / 10.0,
            -std::f32::consts::FRAC_PI_4,
        );
    }
}
