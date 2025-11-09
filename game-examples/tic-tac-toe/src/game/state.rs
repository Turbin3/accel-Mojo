use crate::game::game_logic::{game, Mark};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct StateInfo {
    pub game: game::Game,
    pub current_player: Mark,
    pub computer_thinking_time: Timer,
}

pub fn start_x_turn(mut info: ResMut<StateInfo>) {
    info.current_player = Mark::X
}

pub fn start_o_turn(mut info: ResMut<StateInfo>) {
    info.current_player = Mark::O
}
