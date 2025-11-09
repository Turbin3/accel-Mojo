use bevy::prelude::Component;

#[derive(Component, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Mark {
    #[default]
    X,
    O,
}

impl std::fmt::Display for Mark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mark::X => write!(f, "X"),
            Mark::O => write!(f, "O"),
        }
    }
}

impl Mark {
    pub fn color(&self) -> bevy::prelude::Color {
        match self {
            Mark::X => bevy::prelude::Color::srgb(1.0, 0.0, 0.0), // Red
            Mark::O => bevy::prelude::Color::srgb(0.0, 0.0, 1.0), // Blue
        }
    }

    pub fn is_human(&self) -> bool {
        // Human always plays as X
        matches!(self, Mark::X)
    }
}

// Game is inside game module so private fields of Game cannot be accessed / mutated directly
pub mod game {
    use std::collections::{HashMap, HashSet};

    use super::super::grid::{Cell, Column, Dimension, Line, Row};
    use super::Mark;

    // All of Game's fields are private so that we can recalculate the winner when a new mark is made on the board
    // impl Default is required for impl Default on StateInfo
    #[derive(Default)]
    pub struct Game {
        marks: HashMap<Cell, Option<Mark>>,
        winner: Option<(Mark, Line)>,
        over: bool,
    }

    impl Game {
        const WINNING_ARRANGEMENTS: [(fn(&(&Cell, &Option<Mark>)) -> bool, Line); 8] = [
            (|(cell, _)| cell.row() == Row::Top, Line::TopRow),
            (|(cell, _)| cell.row() == Row::Middle, Line::MiddleRow),
            (|(cell, _)| cell.row() == Row::Bottom, Line::BottomRow),
            (|(cell, _)| cell.column() == Column::Left, Line::LeftColumn),
            (
                |(cell, _)| cell.column() == Column::Middle,
                Line::MiddleColumn,
            ),
            (
                |(cell, _)| cell.column() == Column::Right,
                Line::RightColumn,
            ),
            (
                |(cell, _)| cell.column().position() == cell.row().position(),
                Line::UpDiagonal,
            ),
            (
                |(cell, _)| cell.column().position() == -cell.row().position(),
                Line::DownDiagonal,
            ),
        ];

        fn determine_winner(marks: &HashMap<Cell, Option<Mark>>) -> Option<(Mark, Line)> {
            for (arrangement, line) in Self::WINNING_ARRANGEMENTS {
                let marks = marks
                    .iter()
                    .filter(arrangement)
                    .flat_map(|(_, mark)| *mark)
                    .collect::<Vec<Mark>>();

                let unique_marks = marks.iter().cloned().collect::<HashSet<Mark>>();

                if marks.len() == 3 && unique_marks.len() == 1 {
                    return Some((*marks.get(0).unwrap(), line));
                };
            }

            None
        }

        // behind a getter so the user cannot mutate this field directly
        pub fn winner(&self) -> Option<(Mark, Line)> {
            self.winner
        }

        // behind a getter so the user cannot mutate this field directly
        pub fn over(&self) -> bool {
            self.over
        }

        // behind a getter so the user cannot access / mutate marks directly
        pub fn get(&self, cell: Cell) -> Option<Mark> {
            self.marks.get(&cell).cloned().flatten()
        }

        // behind a setter so we can recalculate the winner immediately
        pub fn set(&mut self, cell: Cell, mark: Mark) {
            self.marks.insert(cell, Some(mark));
            self.winner = Game::determine_winner(&self.marks);
            self.over = self.winner.is_some() || self.marks.len() == 9;
        }
    }
}
