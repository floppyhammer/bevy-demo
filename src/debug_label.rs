use bevy::prelude::*;
use std::time::Duration;
use bevy::text::DEFAULT_FONT_HANDLE;

#[derive(Component)]
struct DebugLabel {
    frame_count: u64,
    timer: Timer,
}

fn setup_debug_label(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(TextBundle {
            text: Text::from_section(
                "",
                {
                    let mut text_style = TextStyle::default();
                    text_style.font_size = 24.0;
                    text_style
                },
            ),
            transform: Transform::default(),
            style: Style::default(),
            ..Default::default()
        })
        .insert(DebugLabel {
            frame_count: 0,
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        });
}

fn update_debug_label(time: Res<Time>, mut query: Query<(&mut DebugLabel, &mut Text)>) {
    for (mut debug_label, mut text) in query.iter_mut() {
        debug_label.frame_count += 1;

        // Update timer.
        debug_label.timer.tick(time.delta());

        if debug_label.timer.finished() {
            text.sections[0].value = format!(
                "FPS {:.0}",
                (debug_label.frame_count as f64 / debug_label.timer.duration().as_secs_f64())
            );

            debug_label.frame_count = 0;
        }
    }
}

pub struct DebugLabelPlugin;

impl Plugin for DebugLabelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_debug_label);
        app.add_systems(Update, update_debug_label);
    }
}
