use bracket_lib::prelude::Point;

pub trait Map {
    fn dimensions(&self) -> Point;

    fn xy_idx(&self, pos: Point) -> usize {
        ((pos.y * self.dimensions().x) + pos.x) as usize
    }

    fn idx_xy(&self, idx: usize) -> Point {
        let width = self.dimensions().x;
        let x = (idx as i32) % width;
        let y = (idx as i32) / width;
        Point { x, y }
    }
}
