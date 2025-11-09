use bevy::prelude::*;

mod ball;
mod collision;
mod components;
mod gameplay;
mod gutter;
mod paddle;
mod scoreboard;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Pong".to_string(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(scoreboard::Score::default())
        .add_systems(
            Startup,
            (
                ball::spawn_ball,
                spawn_camera,
                paddle::spawn_paddles,
                gutter::spawn_gutters,
                scoreboard::spawn_scoreboard,
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                components::project_positions,
                ball::move_ball.before(components::project_positions),
                collision::handle_collisions.after(ball::move_ball),
                paddle::move_paddles.before(components::project_positions),
                paddle::handle_player_input.before(paddle::move_paddles),
                paddle::constrain_paddle_position.after(paddle::move_paddles),
                gameplay::detect_goal.after(ball::move_ball),
                scoreboard::update_scoreboard,
                paddle::move_computer,
            ),
        )
        .add_observer(gameplay::reset_ball)
        .add_observer(gameplay::update_score)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0., 0., 0.)));
}
