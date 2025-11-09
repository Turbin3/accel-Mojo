use std::time::Duration;

use bevy::prelude::*;
use rand::prelude::*;

use super::game_logic::{game, Mark};
use super::grid::{Cell, GameState, CELL_VARIANTS, LINE_VARIANTS};
use super::state::StateInfo;

pub fn capture_user_input(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    touch_input: Res<Touches>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) -> Option<Cell> {
    // expect() because we spawn only a single Camera2dBundle and expect Bevy to be able to provide it to us
    let (camera, camera_transform) = cameras.single().expect("expected exactly one camera");

    // get touch input from users on mobile
    let maybe_touch_coordinates: Option<Vec2> = touch_input
        .iter()
        .filter(|finger| touch_input.just_pressed(finger.id()))
        .next()
        .map(|finger| finger.position());

    // get mouse input from users on desktop
    let maybe_click_coordinates: Option<Vec2> = windows
        .single()
        .iter()
        .filter(|_| mouse_button_input.just_pressed(MouseButton::Left))
        .next()
        .and_then(|window| window.cursor_position());

    maybe_touch_coordinates
        .or(maybe_click_coordinates)
        .and_then(|window_coordinates| {
            camera
                .viewport_to_world_2d(camera_transform, window_coordinates)
                .ok()
        })
        .and_then(|world_coordinates| Cell::hit(world_coordinates))
}

pub fn generate_computer_input(game: &game::Game, computer: Mark) -> Cell {
    // weight cells based on their advantage to the computer and their disadvantage to the human
    //
    //   1. +20 for any cell which lets the computer win this turn
    //   2. +10 for any cell which blocks a human win this turn
    //   3. +2 for the middle-middle space
    //   4. +1 for any corner space
    //
    // ...then, just pick the cell with the highest weight, after filtering out already-occupied cells

    let mut weights: [i8; 9] = [0, 0, 0, 0, 0, 0, 0, 0, 0];

    // Always use medium difficulty - randomly pick best-possible and worst-possible moves
    let mut rng = rand::rng();
    let scale = *[-1, 1]
        .choose(&mut rng)
        .expect("array is non-empty, so we should always get a value");

    fn index(cell: Cell) -> usize {
        match cell {
            Cell::TopLeft => 0,
            Cell::TopMiddle => 1,
            Cell::TopRight => 2,
            Cell::MiddleLeft => 3,
            Cell::MiddleMiddle => 4,
            Cell::MiddleRight => 5,
            Cell::BottomLeft => 6,
            Cell::BottomMiddle => 7,
            Cell::BottomRight => 8,
        }
    }

    LINE_VARIANTS.iter().for_each(|line| {
        let cells_and_marks = line.cells().map(|cell| (cell, game.get(cell)));

        // case (1)
        match cells_and_marks {
            [(_, Some(a)), (_, Some(b)), (cell, None)] if a == b && b == computer => {
                weights[index(cell)] += 20 * scale
            }
            [(_, Some(a)), (cell, None), (_, Some(b))] if a == b && b == computer => {
                weights[index(cell)] += 20 * scale
            }
            [(cell, None), (_, Some(a)), (_, Some(b))] if a == b && b == computer => {
                weights[index(cell)] += 20 * scale
            }
            _ => {}
        }

        // case (2)
        match cells_and_marks {
            [(_, Some(a)), (_, Some(b)), (cell, None)] if a == b && b != computer => {
                weights[index(cell)] += 10 * scale
            }
            [(_, Some(a)), (cell, None), (_, Some(b))] if a == b && b != computer => {
                weights[index(cell)] += 10 * scale
            }
            [(cell, None), (_, Some(a)), (_, Some(b))] if a == b && b != computer => {
                weights[index(cell)] += 10 * scale
            }
            _ => {}
        }

        // case (3)
        match cells_and_marks {
            [_, (cell, None), _] if cell == Cell::MiddleMiddle => weights[index(cell)] += 2 * scale,
            _ => {}
        }

        // case (4)
        match cells_and_marks {
            [(c1, None), _, (c2, None)] if c1.is_corner() => {
                weights[index(c1)] += 1 * scale;
                weights[index(c2)] += 1 * scale
            }
            [(cell, None), _, _] if cell.is_corner() => weights[index(cell)] += 1 * scale,
            [_, _, (cell, None)] if cell.is_corner() => weights[index(cell)] += 1 * scale,
            _ => {}
        }
    });

    info!("cell weights (higher is better): {:?}", weights);

    let (index, _) = weights
        .iter()
        .enumerate()
        .filter(|(index, _)| game.get(CELL_VARIANTS[*index]).is_none())
        .max_by(|&(_, w1), (_, w2)| w1.cmp(w2))
        .expect("unable to find max weight");

    let chosen_cell = CELL_VARIANTS[index];

    info!("optimal cell for computer to choose is {:?}", chosen_cell);

    chosen_cell
}

pub fn capture_input(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut info: ResMut<StateInfo>,
    cells: Query<(Entity, &Cell)>,
    touch_input: Res<Touches>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    current_game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    if info.game.over() {
        return;
    }

    let mark = info.current_player;

    let maybe_cell = if !mark.is_human() {
        info.computer_thinking_time.tick(time.delta());

        if info.computer_thinking_time.is_finished() {
            Some(generate_computer_input(&info.game, mark))
        } else {
            None
        }
    } else {
        let user_input = capture_user_input(windows, cameras, touch_input, mouse_button_input);
        info.computer_thinking_time
            .set_duration(Duration::from_millis(400));
        info.computer_thinking_time.reset();
        user_input
    };

    let Some(cell) = maybe_cell else {
        return;
    };

    match info.game.get(cell) {
        Some(_) => warn!("this cell is already occupied"),
        None => {
            let (entity, cell) = cells
                .iter()
                .filter(|(_, c)| c == &&cell)
                .next()
                .expect("could not find clicked cell in all cells");

            info.game.set(*cell, mark);
            info!("{:?} was hit", cell);

            commands.entity(entity).with_children(|parent| {
                parent
                    .spawn((
                        Text::new(mark.to_string()),
                        TextFont {
                            font: Default::default(),
                            font_size: 200.0,
                            ..default()
                        },
                        TextColor(mark.color()),
                    ))
                    .insert(mark);
            });

            if info.game.over() {
                match info.game.winner() {
                    None => {
                        info!("The game ends in a tie");
                    }
                    Some((mark, line)) => {
                        let [from, .., to] = line.cells();
                        info!(
                            "The winner is {} along the line {:?} -> {:?}",
                            mark, from, to
                        );
                    }
                }

                next_game_state.set(GameState::GameOver)
            } else {
                match *current_game_state.get() {
                    GameState::XTurn => next_game_state.set(GameState::OTurn),
                    GameState::OTurn => next_game_state.set(GameState::XTurn),
                    GameState::GameOver => unreachable!("called capture_input() in GameOver state"),
                    GameState::GameNotInProgress => {
                        unreachable!("called capture_input() in GameNotInProgress state")
                    }
                }
            }
        }
    }
}
