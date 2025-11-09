use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const BOX_SIZE: f32 = 60.0;
const BOX_SPEED: f32 = 200.0;
const BOX_COLOR: Color = Color::srgb(0.7, 0.7, 0.0);

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Direction(Vec2);

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
struct BoxIndicator;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let player_entity = commands
        .spawn((
            Sprite {
                color: BOX_COLOR,
                custom_size: Some(Vec2::new(BOX_SIZE, BOX_SIZE)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            Player,
            Velocity(Vec2::ZERO),
            Direction(Vec2::Y),
        ))
        .id();

    // Direction indicator as child entity
    commands.entity(player_entity).with_children(|parent| {
        parent.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(BOX_SIZE / 3.0, BOX_SIZE / 3.0)),
                ..default()
            },
            Transform::from_xyz(0.0, BOX_SIZE / 2.0, 0.1),
            BoxIndicator,
        ));
    });
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Velocity, &mut Direction, &Transform), With<Player>>,
) {
    if let Ok((mut velocity, mut direction, _transform)) = player_query.single_mut() {
        let mut new_velocity = Vec2::ZERO;
        let mut new_direction = direction.0;

        if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
            new_velocity.x = -1.0;
            new_direction = Vec2::new(-1.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
            new_velocity.x = 1.0;
            new_direction = Vec2::new(1.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
            new_velocity.y = 1.0;
            new_direction = Vec2::new(0.0, 1.0);
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
            new_velocity.y = -1.0;
            new_direction = Vec2::new(0.0, -1.0);
        }

        // Normalize to prevent faster diagonal movement
        if new_velocity.length_squared() > 0.0 {
            velocity.0 = new_velocity.normalize();
        } else {
            velocity.0 = Vec2::ZERO;
        }

        if new_direction != Vec2::ZERO {
            direction.0 = new_direction;
        }
    }
}

fn apply_velocity(
    mut player_query: Query<(&mut Transform, &Velocity, &Direction), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, velocity, direction)) = player_query.single_mut() {
        transform.translation.x += velocity.0.x * BOX_SPEED * time.delta_secs();
        transform.translation.y += velocity.0.y * BOX_SPEED * time.delta_secs();

        // Rotate to face direction (subtract 90° so Vec2::Y points up)
        if direction.0 != Vec2::ZERO {
            let angle = direction.0.y.atan2(direction.0.x);
            transform.rotation = Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2);
        }
    }
}

fn confine_player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(mut player_transform) = player_query.single_mut() {
        let window = window_query.single().unwrap();

        let half_box_size = BOX_SIZE / 2.0;
        let x_min = -window.width() / 2.0 + half_box_size;
        let x_max = window.width() / 2.0 - half_box_size;
        let y_min = -window.height() / 2.0 + half_box_size;
        let y_max = window.height() / 2.0 - half_box_size;

        let mut translation = player_transform.translation;

        if translation.x < x_min {
            translation.x = x_min;
        } else if translation.x > x_max {
            translation.x = x_max;
        }

        if translation.y < y_min {
            translation.y = y_min;
        } else if translation.y > y_max {
            translation.y = y_max;
        }

        player_transform.translation = translation;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Moving Box".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (player_movement, apply_velocity, confine_player_movement).chain(),
        )
        .run();
}
