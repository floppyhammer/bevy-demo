use std::time::Duration;
use bevy::prelude::*;

#[derive(Component)]
struct DebugLabel {
    frame_count: u64,
    timer: Timer,
}

fn debug_label_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "Some Text",
            TextStyle {
                font: asset_server.load("fonts/VonwaonBitmap-12px.ttf"),
                font_size: 48.0,
                color: Color::WHITE,
            },
        ),
        transform: Transform::default(),
        ..Default::default()
    }).insert(DebugLabel { frame_count: 0, timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating) });
}

fn debug_label_system(
    time: Res<Time>,
    mut query: Query<(
        &mut DebugLabel,
        &mut Text,
    )>) {
    for (mut debug_label, mut text) in query.iter_mut() {
        debug_label.frame_count += 1;

        // Update timer.
        debug_label.timer.tick(time.delta());

        if debug_label.timer.finished() {
            text.sections[0].value = format!("FPS {:.0}", (debug_label.frame_count as f64 / debug_label.timer.duration().as_secs_f64()));

            debug_label.frame_count = 0;
        }
    }
}

pub struct DebugLabelPlugin;

impl Plugin for DebugLabelPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(debug_label_setup);
        app.add_system(debug_label_system);
    }
}
