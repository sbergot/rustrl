use bracket_lib::prelude::Point;

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
        return if self.0.is_empty() { None } else { Some(self.0[0]) };
    }
}
