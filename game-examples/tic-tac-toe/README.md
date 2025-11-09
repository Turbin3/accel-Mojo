# Mojo Tic Tac Toe

Tic Tac Toe game built with Bevy game engine showcasing Mojo SDK.

## Overview

This is a fully functional Tic Tac Toe game built with the Bevy game engine in Rust. The game features human vs computer gameplay modes.

## Installation

1. Ensure you have Rust and Cargo installed on your system
2. Clone this repository
3. Navigate to the project directory: `cd /path/to/tic-tac-toe`
4. Run the game with: `cargo run`

## How to Play

- Click the "Start Game" button on the main menu
- The game alternates between X and O turns
- Click on any empty cell to place your mark
- The first player to get 3 of their marks in a row (horizontally, vertically, or diagonally) wins
- The game ends when a player wins or all cells are filled (tie)

## Technical Details

- Built with Bevy 0.17.2 game engine
- Uses Bevy's ECS (Entity Component System) architecture

## Project Structure

- `src/main.rs` - Main application entry point and setup
- `src/menu.rs` - Menu screen with start button
- `src/game/` - Core game functionality:
  - `game_logic.rs` - Tic Tac Toe game rules and logic
  - `grid.rs` - Grid representation and state management
  - `input.rs` - Input handling for both human and computer players
  - `state.rs` - Game state management
  - `ui.rs` - User interface elements and game display

## Acknowledgements

This game was adapted from the Tic Tac Toe at <https://github.com/awwsmm/tic-tac-toe>. The original project served as a foundation for this Bevy-based implementation.
