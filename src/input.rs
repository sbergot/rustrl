use bracket_lib::prelude::{VirtualKeyCode, Point};

#[derive(Clone, Copy)]
pub enum Command {
    Direction { direction: Direction },
    Grab,
    ShowInventory,
    ShowDropItem,
    ShowRemoveItem,
    ExamineMode,
    SaveQuit,
    Validate,
    Cancel,
    NextTarget,
}

#[derive(Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

pub fn map_direction(key: VirtualKeyCode) -> Option<Command> {
    match key {
        VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
            Some(Command::Direction {
                direction: Direction::Left,
            })
        }
        VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
            Some(Command::Direction {
                direction: Direction::Right,
            })
        }
        VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
            Some(Command::Direction {
                direction: Direction::Up,
            })
        }
        VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
            Some(Command::Direction {
                direction: Direction::Down,
            })
        }
        VirtualKeyCode::Y => Some(Command::Direction {
            direction: Direction::UpLeft,
        }),
        VirtualKeyCode::U => Some(Command::Direction {
            direction: Direction::UpRight,
        }),
        VirtualKeyCode::B => Some(Command::Direction {
            direction: Direction::DownLeft,
        }),
        VirtualKeyCode::N => Some(Command::Direction {
            direction: Direction::DownRight,
        }),
        _ => None,
    }
}

pub fn map_look_commands(key: VirtualKeyCode) -> Option<Command> {
    match key {
        VirtualKeyCode::Tab => Some(Command::NextTarget),
        VirtualKeyCode::Return => Some(Command::Validate),
        VirtualKeyCode::Escape => Some(Command::Cancel),
        _ => None,
    }
}

pub fn map_other_commands(key: VirtualKeyCode) -> Option<Command> {
    match key {
        VirtualKeyCode::G => Some(Command::Grab),
        VirtualKeyCode::I => Some(Command::ShowInventory),
        VirtualKeyCode::D => Some(Command::ShowDropItem),
        VirtualKeyCode::R => Some(Command::ShowRemoveItem),
        VirtualKeyCode::X => Some(Command::ExamineMode),
        VirtualKeyCode::Escape => Some(Command::SaveQuit),
        _ => None,
    }
}

pub fn map_all(key: Option<VirtualKeyCode>, mappings: &[fn(key: VirtualKeyCode) -> Option<Command>]) -> Option<Command> {
    match key {
        None => None,
        Some(key) => {
            for mapping in mappings {
                if let Some(cmd) = mapping(key) {
                    return Some(cmd);
                }
            }
            None
        }
    }
}

pub fn get_direction_offset(direction: Direction) -> Point {
    match direction {
        Direction::Left => Point { x: -1, y: 0 },
        Direction::Right => Point { x: 1, y: 0 },
        Direction::Up => Point { x: 0, y: -1 },
        Direction::Down => Point { x: 0, y: 1 },
        Direction::UpLeft => Point { x: -1, y: -1 },
        Direction::UpRight => Point { x: 1, y: -1 },
        Direction::DownLeft => Point { x: -1, y: 1 },
        Direction::DownRight => Point { x: 1, y: 1 },
    }
}