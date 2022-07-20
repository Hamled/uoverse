use serde_repr::{Deserialize_repr, Serialize_repr};

use super::Direction;

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum MovementRaw {
    North = 0,
    Right,
    East,
    Down,
    South,
    Left,
    West,
    Up,

    RunNorth = 0x80,
    RunRight,
    RunEast,
    RunDown,
    RunSouth,
    RunLeft,
    RunWest,
    RunUp,
}

impl From<Direction> for MovementRaw {
    fn from(val: Direction) -> Self {
        use Direction::*;

        match val {
            North => Self::North,
            Right => Self::Right,
            East => Self::East,
            Down => Self::Down,
            South => Self::South,
            Left => Self::Left,
            West => Self::West,
            Up => Self::Up,
        }
    }
}

impl From<Movement> for MovementRaw {
    fn from(val: Movement) -> MovementRaw {
        use Direction::*;

        let Movement { dir, run } = val;
        match (dir, run) {
            (North, false) => Self::North,
            (Right, false) => Self::Right,
            (East, false) => Self::East,
            (Down, false) => Self::Down,
            (South, false) => Self::South,
            (Left, false) => Self::Left,
            (West, false) => Self::West,
            (Up, false) => Self::Up,
            (North, true) => Self::RunNorth,
            (Right, true) => Self::RunRight,
            (East, true) => Self::RunEast,
            (Down, true) => Self::RunDown,
            (South, true) => Self::RunSouth,
            (Left, true) => Self::RunLeft,
            (West, true) => Self::RunWest,
            (Up, true) => Self::RunUp,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Movement {
    pub dir: Direction,
    pub run: bool,
}

impl From<Direction> for Movement {
    fn from(val: Direction) -> Self {
        Self {
            dir: val,
            run: false,
        }
    }
}

impl From<(Direction, bool)> for Movement {
    fn from((dir, run): (Direction, bool)) -> Self {
        Self { dir, run }
    }
}

impl From<MovementRaw> for Movement {
    fn from(val: MovementRaw) -> Self {
        let dir: Direction = val.into();
        let run = (val as u8) >= (MovementRaw::RunNorth as u8);

        Self { dir, run }
    }
}

impl From<MovementRaw> for Direction {
    fn from(val: MovementRaw) -> Self {
        use MovementRaw::*;

        match val {
            North => Self::North,
            Right => Self::Right,
            East => Self::East,
            Down => Self::Down,
            South => Self::South,
            Left => Self::Left,
            West => Self::West,
            Up => Self::Up,
            RunNorth => Self::North,
            RunRight => Self::Right,
            RunEast => Self::East,
            RunDown => Self::Down,
            RunSouth => Self::South,
            RunLeft => Self::Left,
            RunWest => Self::West,
            RunUp => Self::Up,
        }
    }
}

impl From<Movement> for Direction {
    fn from(val: Movement) -> Self {
        val.dir
    }
}
