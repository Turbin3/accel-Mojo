use bevy::prelude::*;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::ball::{BALL_SPEED, Ball};
use crate::components::{Collider, Computer, Player, Position, Velocity};
use crate::scoreboard::Score;

#[derive(EntityEvent)]
pub struct Scored {
    #[event_target]
    pub scorer: Entity,
}

pub fn detect_goal(
    ball: Single<(&Position, &Collider), With<Ball>>,
    player: Single<Entity, (With<Player>, Without<Computer>)>,
    computer: Single<Entity, (With<Computer>, Without<Player>)>,
    window: Single<&Window>,
    mut commands: Commands,
) {
    let (ball_position, ball_collider) = ball.into_inner();
    let half_window_size = window.resolution.size() / 2.;

    if ball_position.0.x - ball_collider.half_size().x > half_window_size.x {
        commands.trigger(Scored { scorer: *player });
    }
    if ball_position.0.x + ball_collider.half_size().x < -half_window_size.x {
        commands.trigger(Scored { scorer: *computer });
    }
}

pub fn reset_ball(_event: On<Scored>, ball: Single<(&mut Position, &mut Velocity), With<Ball>>) {
    let (mut ball_position, mut ball_velocity) = ball.into_inner();

    ball_position.0 = Vec2::ZERO;

    let mut rng: ThreadRng = rand::rng();
    let random_y = rng.random_range(-1.0..1.0); // Random y direction between -1 and 1
    let random_x_sign = if rng.random_bool(0.5) { 1.0 } else { -1.0 }; // Randomly start left or right

    ball_velocity.0 =
        Vec2::new(BALL_SPEED * random_x_sign, BALL_SPEED * random_y).normalize() * BALL_SPEED;
}

pub fn update_score(
    event: On<Scored>,
    mut score: ResMut<Score>,
    is_player: Query<&Player>,
    is_computer: Query<&Computer>,
) {
    if is_computer.get(event.scorer).is_ok() {
        score.computer += 1;
        info!("Computer Scored! {} - {}", score.player, score.computer);
    }
    if is_player.get(event.scorer).is_ok() {
        score.player += 1;
        info!("Player Scored! {} - {}", score.player, score.computer);
    }
}
