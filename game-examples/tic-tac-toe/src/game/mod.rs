mod game_logic;
mod grid;
mod input;
mod state;
mod ui;

use crate::{clear_entities, AppState};
use bevy::prelude::*;
use game_logic::Mark;
use input::capture_input;
use state::{start_o_turn, start_x_turn, StateInfo};
use ui::{game_over, game_over_buttons, start_game};

pub fn plugin(app: &mut App) {
    app.insert_resource(StateInfo::default())
        .add_systems(OnEnter(AppState::Game), start_game)
        .init_state::<grid::GameState>()
        .add_systems(OnEnter(grid::GameState::XTurn), start_x_turn)
        .add_systems(
            Update,
            capture_input.run_if(in_state(grid::GameState::XTurn)),
        )
        .add_systems(OnEnter(grid::GameState::OTurn), start_o_turn)
        .add_systems(
            Update,
            capture_input.run_if(in_state(grid::GameState::OTurn)),
        )
        .add_systems(OnEnter(grid::GameState::GameOver), game_over)
        .add_systems(
            Update,
            game_over_buttons.run_if(in_state(grid::GameState::GameOver)),
        )
        .add_systems(OnExit(grid::GameState::GameOver), clear_entities::<Mark>)
        .add_systems(
            OnExit(grid::GameState::GameOver),
            clear_entities::<ui::GameOverOverlay>,
        )
        .add_systems(OnExit(AppState::Game), clear_entities::<AppState>)
        .add_systems(
            OnExit(AppState::Game),
            clear_entities::<ui::GameOverOverlay>,
        );
}
