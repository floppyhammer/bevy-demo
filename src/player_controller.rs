use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct PlayerController;

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_controller_system);
    }
}

// fn player_controller_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//     commands.spawn(Text2dBundle {
//         text: Text::from_section(
//             "Some Text",
//             TextStyle {
//                 font: asset_server.load("fonts/VonwaonBitmap-12px.ttf"),
//                 font_size: 48.0,
//                 color: Color::WHITE,
//             },
//         ),
//         transform: Transform::default(),
//         ..Default::default()
//     });
// }

fn player_controller_system(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<PlayerController>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let mut dir = Vec2::new(0.0, 0.0);

    if keyboard_input.pressed(KeyCode::Right) {
        dir.x = 1.0;
    } else if keyboard_input.pressed(KeyCode::Left) {
        dir.x = -1.0;
    }
    if keyboard_input.pressed(KeyCode::Up) {
        dir.y = 1.0;
    } else if keyboard_input.pressed(KeyCode::Down) {
        dir.y = -1.0;
    }

    if dir.length() != 0.0 {
        dir = dir.normalize();
    }

    let speed = 100.0;

    // `Transform.translation` will determine the location of the text.
    // `Transform.scale` and `Transform.rotation` do not yet affect text (though you can set the
    // size of the text via `Text.style.font_size`)
    for mut transform in query.iter_mut() {
        transform.translation.x += dir.x * speed * time.delta_seconds();
        transform.translation.y += dir.y * speed * time.delta_seconds();
    }
}
