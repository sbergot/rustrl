use bracket_lib::prelude::Point;
use specs::Entity;

pub struct PlayerPos {
    pub pos: Point,
}

pub struct PlayerEntity {
    pub entity: Entity,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

pub struct PointsOfInterest(Vec<Point>);

impl PointsOfInterest {
    pub fn new() -> PointsOfInterest {
        PointsOfInterest(Vec::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn add(&mut self, point: Point) {
        self.0.push(point);
    }

    pub fn get_next(&self, prev: Point) -> Option<Point> {
        for (idx, poi) in self.0.iter().enumerate() {
            if *poi == prev {
                return Some(self.0[(idx + 1) % self.0.len()]);
            }
        }
        return if self.0.is_empty() {
            None
        } else {
            Some(self.0[0])
        };
    }

    pub fn contains(&self, pos: Point) -> bool {
        self.0.contains(&pos)
    }
}
