use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowMode};

/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
pub(crate) struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
    pub window_size: (i32, i32),
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
            window_size: (0, 0),
        }
    }
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
pub(crate) fn pan_orbit_camera(
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
    window_query: Query<&Window>,
) {
    let window = window_query.single();
    let window_size = Vec2::new(window.width(), window.height());

    // Change input mapping for orbit and panning here.
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.iter() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.iter() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        let projection = match projection {
            Projection::Perspective(pp) => pp,
            _ => panic!(),
        };

        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let delta_x = {
                let delta = rotation_move.x / window_size.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window_size.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // Make panning distance independent of resolution and FOV.
            pan *=
                Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window_size;
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }
}

pub(crate) fn move_camera_by_keyboard(
    time: Res<Time>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform)>,
    window_query: Query<&Window>,
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

    let speed = 0.1;

    // `Transform.translation` will determine the location of the text.
    // `Transform.scale` and `Transform.rotation` do not yet affect text (though you can set the
    // size of the text via `Text.style.font_size`)
    for (_, mut transform) in query.iter_mut() {
        transform.translation.x += dir.x * speed * time.delta_seconds();
        transform.translation.y += dir.y * speed * time.delta_seconds();
    }
}
