use bevy::prelude::*;

use crate::ball::Ball;
use crate::collision::{collide_with_side, Collision};
use crate::components::{Collider, Computer, Player, Position, Velocity};
use crate::gutter::Gutter;
use bevy::math::bounding::Aabb2d;

#[derive(Component)]
#[require(Position,
    Collider = Collider(PADDLE_SHAPE),
    Velocity,
)]
pub struct Paddle;

pub const PADDLE_SHAPE: Rectangle = Rectangle::new(20., 50.);
pub const PLAYER_PADDLE_COLOR: Color = Color::srgb(0., 0., 1.);
pub const COMPUTER_PADDLE_COLOR: Color = Color::srgb(1., 0., 0.);
pub const PADDLE_SPEED: f32 = 5.;

pub fn spawn_paddles(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    let mesh = meshes.add(PADDLE_SHAPE);
    let player_material = materials.add(PLAYER_PADDLE_COLOR);
    let computer_material = materials.add(COMPUTER_PADDLE_COLOR);

    let half_window_size = window.resolution.size() / 2.;
    let paddle_padding = 20.;

    let player_position = Vec2::new(-half_window_size.x + paddle_padding, 0.);

    commands.spawn((
        Player,
        Paddle,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(player_material.clone()),
        Position(player_position),
    ));

    let computer_position = Vec2::new(half_window_size.x - paddle_padding, 0.);

    commands.spawn((
        Computer,
        Paddle,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(computer_material.clone()),
        Position(computer_position),
    ));
}

pub fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut paddle_velocity: Single<&mut Velocity, With<Player>>,
) {
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        paddle_velocity.0.y = PADDLE_SPEED;
    } else if keyboard_input.pressed(KeyCode::ArrowDown) {
        paddle_velocity.0.y = -PADDLE_SPEED;
    } else {
        paddle_velocity.0.y = 0.;
    }
}

pub fn move_paddles(mut paddles: Query<(&mut Position, &Velocity), With<Paddle>>) {
    for (mut position, velocity) in &mut paddles {
        position.0 += velocity.0;
    }
}

pub fn constrain_paddle_position(
    mut paddles: Query<(&mut Position, &Collider), (With<Paddle>, Without<Gutter>)>,
    gutters: Query<(&Position, &Collider), (With<Gutter>, Without<Paddle>)>,
) {
    for (mut paddle_position, paddle_collider) in &mut paddles {
        for (gutter_position, gutter_collider) in &gutters {
            let paddle_aabb = Aabb2d::new(paddle_position.0, paddle_collider.half_size());
            let gutter_aabb = Aabb2d::new(gutter_position.0, gutter_collider.half_size());

            if let Some(collision) = collide_with_side(paddle_aabb, gutter_aabb) {
                match collision {
                    Collision::Top => {
                        paddle_position.0.y = gutter_position.0.y
                            + gutter_collider.half_size().y
                            + paddle_collider.half_size().y;
                    }
                    Collision::Bottom => {
                        paddle_position.0.y = gutter_position.0.y
                            - gutter_collider.half_size().y
                            - paddle_collider.half_size().y;
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn move_computer(
    computer: Single<(&mut Velocity, &Position), With<Computer>>,
    ball: Single<&Position, With<Ball>>,
) {
    let (mut velocity, position) = computer.into_inner();

    let a_to_b = ball.0 - position.0;
    velocity.0.y = a_to_b.y.signum() * PADDLE_SPEED;
}
