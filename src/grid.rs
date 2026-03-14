//! Flat 2D grid backed by a `Vec<T>`.

use crate::types::Pos;

/// 4-connected neighbor offsets (N, E, S, W).
const DIRS_4: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// A 2D grid stored as a flat `Vec<T>` in row-major order.
#[derive(Clone, Debug)]
pub struct Grid<T> {
    pub width: u32,
    pub height: u32,
    cells: Vec<T>,
}

impl<T: Clone> Grid<T> {
    /// Create a grid filled with `value`.
    pub fn new(width: u32, height: u32, value: T) -> Self {
        Self {
            width,
            height,
            cells: vec![value; (width * height) as usize],
        }
    }
}

impl<T> Grid<T> {
    /// Returns true if `(x, y)` is within bounds.
    pub fn in_bounds(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }

    fn index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn get(&self, x: u32, y: u32) -> Option<&T> {
        if self.in_bounds(x, y) {
            Some(&self.cells[self.index(x, y)])
        } else {
            None
        }
    }

    pub fn get_pos(&self, pos: Pos) -> Option<&T> {
        self.get(pos.x, pos.y)
    }

    pub fn set(&mut self, x: u32, y: u32, value: T) {
        if self.in_bounds(x, y) {
            let i = self.index(x, y);
            self.cells[i] = value;
        }
    }

    /// 4-connected neighbors within bounds (collected to avoid borrow issues).
    pub fn neighbors4(&self, x: u32, y: u32) -> Vec<Pos> {
        let w = self.width;
        let h = self.height;
        DIRS_4
            .into_iter()
            .filter_map(|(dx, dy)| {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as u32) < w && (ny as u32) < h {
                    Some(Pos::new(nx as u32, ny as u32))
                } else {
                    None
                }
            })
            .collect()
    }

    /// All positions in the grid (collected to avoid borrow issues).
    pub fn positions(&self) -> Vec<Pos> {
        (0..self.height)
            .flat_map(|y| (0..self.width).map(move |x| Pos::new(x, y)))
            .collect()
    }
}
