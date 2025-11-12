use bevy::prelude::*;
use bevy::math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume};

use crate::ball::Ball;
use crate::components::{Collider, Position, Velocity};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

pub fn collide_with_side(ball: Aabb2d, wall: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&wall) {
        return None;
    }

    let closest_point = wall.closest_point(ball.center());
    let offset = ball.center() - closest_point;

    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

pub fn handle_collisions(
    ball: Single<(&mut Velocity, &Position, &Collider), With<Ball>>,
    other_things: Query<(&Position, &Collider), Without<Ball>>,
) {
    let (mut ball_velocity, ball_position, ball_collider) = ball.into_inner();

    for (other_position, other_collider) in &other_things {
        if let Some(collision) = collide_with_side(
            Aabb2d::new(ball_position.0, ball_collider.half_size()),
            Aabb2d::new(other_position.0, other_collider.half_size()),
        ) {
            match collision {
                Collision::Left => {
                    ball_velocity.0.x *= -1.;
                }
                Collision::Right => {
                    ball_velocity.0.x *= -1.;
                }
                Collision::Top => {
                    ball_velocity.0.y *= -1.;
                }
                Collision::Bottom => {
                    ball_velocity.0.y *= -1.;
                }
            }
        }
    }
}
