use bevy::math::Vec2;
use bevy::prelude::*;

// Define the GRID_SPACING constant used by the dimension calculations
pub const GRID_SPACING: f32 = 250.0;
pub const HALFSIZE: f32 = GRID_SPACING / 2.0;

// Trait that provides grid positioning functionality
pub trait Dimension {
    fn position(&self) -> i8;
    fn range(&self) -> Vec2;
    fn in_range(&self, value: f32) -> bool;
    fn containing(value: f32) -> Option<Self>
    where
        Self: Copy + PartialEq;
}

#[derive(States, Clone, Hash, PartialEq, Eq, Debug, Default)]
pub enum GameState {
    #[default]
    GameNotInProgress,
    XTurn,
    OTurn,
    GameOver,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Component)]
pub enum Row {
    Bottom,
    Middle,
    Top,
}

impl Dimension for Row {
    fn position(&self) -> i8 {
        match self {
            Row::Top => -1,
            Row::Middle => 0,
            Row::Bottom => 1,
        }
    }

    fn range(&self) -> Vec2 {
        match self {
            Row::Top => Vec2::new(-3.0 * HALFSIZE, -HALFSIZE),
            Row::Middle => Vec2::new(-HALFSIZE, HALFSIZE),
            Row::Bottom => Vec2::new(HALFSIZE, 3.0 * HALFSIZE),
        }
    }

    fn in_range(&self, value: f32) -> bool {
        let Vec2 { x: min, y: max } = self.range();
        min <= value && value < max
    }

    fn containing(value: f32) -> Option<Self>
    where
        Self: Copy + PartialEq,
    {
        if Row::Top.in_range(value) {
            Some(Row::Top)
        } else if Row::Middle.in_range(value) {
            Some(Row::Middle)
        } else if Row::Bottom.in_range(value) {
            Some(Row::Bottom)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Component)]
pub enum Column {
    Left,
    Middle,
    Right,
}

impl Dimension for Column {
    fn position(&self) -> i8 {
        match self {
            Column::Left => -1,
            Column::Middle => 0,
            Column::Right => 1,
        }
    }

    fn range(&self) -> Vec2 {
        match self {
            Column::Left => Vec2::new(-3.0 * HALFSIZE, -HALFSIZE),
            Column::Middle => Vec2::new(-HALFSIZE, HALFSIZE),
            Column::Right => Vec2::new(HALFSIZE, 3.0 * HALFSIZE),
        }
    }

    fn in_range(&self, value: f32) -> bool {
        let Vec2 { x: min, y: max } = self.range();
        min <= value && value < max
    }

    fn containing(value: f32) -> Option<Self>
    where
        Self: Copy + PartialEq,
    {
        if Column::Left.in_range(value) {
            Some(Column::Left)
        } else if Column::Middle.in_range(value) {
            Some(Column::Middle)
        } else if Column::Right.in_range(value) {
            Some(Column::Right)
        } else {
            None
        }
    }
}

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Cell {
    TopLeft,
    TopMiddle,
    TopRight,
    MiddleLeft,
    MiddleMiddle,
    MiddleRight,
    BottomLeft,
    BottomMiddle,
    BottomRight,
}

impl Cell {
    pub fn row(&self) -> Row {
        match self {
            Cell::TopLeft => Row::Top,
            Cell::TopMiddle => Row::Top,
            Cell::TopRight => Row::Top,
            Cell::MiddleLeft => Row::Middle,
            Cell::MiddleMiddle => Row::Middle,
            Cell::MiddleRight => Row::Middle,
            Cell::BottomLeft => Row::Bottom,
            Cell::BottomMiddle => Row::Bottom,
            Cell::BottomRight => Row::Bottom,
        }
    }

    pub fn column(&self) -> Column {
        match self {
            Cell::TopLeft => Column::Left,
            Cell::TopMiddle => Column::Middle,
            Cell::TopRight => Column::Right,
            Cell::MiddleLeft => Column::Left,
            Cell::MiddleMiddle => Column::Middle,
            Cell::MiddleRight => Column::Right,
            Cell::BottomLeft => Column::Left,
            Cell::BottomMiddle => Column::Middle,
            Cell::BottomRight => Column::Right,
        }
    }

    pub fn from(row: Row, column: Column) -> Cell {
        match row {
            Row::Bottom => match column {
                Column::Left => Cell::BottomLeft,
                Column::Middle => Cell::BottomMiddle,
                Column::Right => Cell::BottomRight,
            },
            Row::Middle => match column {
                Column::Left => Cell::MiddleLeft,
                Column::Middle => Cell::MiddleMiddle,
                Column::Right => Cell::MiddleRight,
            },
            Row::Top => match column {
                Column::Left => Cell::TopLeft,
                Column::Middle => Cell::TopMiddle,
                Column::Right => Cell::TopRight,
            },
        }
    }

    pub fn is_corner(&self) -> bool {
        *self == Self::TopLeft
            || *self == Self::TopRight
            || *self == Self::BottomLeft
            || *self == Self::BottomRight
    }

    pub fn hit(pos: Vec2) -> Option<Cell> {
        match (Row::containing(pos.y), Column::containing(pos.x)) {
            (None, _) | (_, None) => None,
            (Some(row), Some(col)) => Some(Cell::from(row, col)),
        }
    }
}

pub const CELL_VARIANTS: [Cell; 9] = [
    Cell::TopLeft,
    Cell::TopMiddle,
    Cell::TopRight,
    Cell::MiddleLeft,
    Cell::MiddleMiddle,
    Cell::MiddleRight,
    Cell::BottomLeft,
    Cell::BottomMiddle,
    Cell::BottomRight,
];

#[derive(Clone, Copy)]
pub enum Line {
    BottomRow,
    MiddleRow,
    TopRow,
    LeftColumn,
    MiddleColumn,
    RightColumn,
    UpDiagonal,
    DownDiagonal,
}

impl Line {
    pub fn cells(&self) -> [Cell; 3] {
        match self {
            Self::BottomRow => [Cell::BottomLeft, Cell::BottomMiddle, Cell::BottomRight],
            Self::MiddleRow => [Cell::MiddleLeft, Cell::MiddleMiddle, Cell::MiddleRight],
            Self::TopRow => [Cell::TopLeft, Cell::TopMiddle, Cell::TopRight],
            Self::LeftColumn => [Cell::TopLeft, Cell::MiddleLeft, Cell::BottomLeft],
            Self::MiddleColumn => [Cell::TopMiddle, Cell::MiddleMiddle, Cell::BottomMiddle],
            Self::RightColumn => [Cell::TopRight, Cell::MiddleRight, Cell::BottomRight],
            Self::UpDiagonal => [Cell::BottomLeft, Cell::MiddleMiddle, Cell::TopRight],
            Self::DownDiagonal => [Cell::TopLeft, Cell::MiddleMiddle, Cell::BottomRight],
        }
    }
}

pub const LINE_VARIANTS: [Line; 8] = [
    Line::BottomRow,
    Line::MiddleRow,
    Line::TopRow,
    Line::LeftColumn,
    Line::MiddleColumn,
    Line::RightColumn,
    Line::UpDiagonal,
    Line::DownDiagonal,
];
