use rand::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Dead,
    Alive,
}

#[derive(Debug)]
pub struct GameOfLife {
    cells: Vec<Vec<Cell>>,
}

impl GameOfLife {
    pub fn new(width: usize, height: usize) -> anyhow::Result<Self> {
        anyhow::ensure!(width > 0 && height > 0, "width and height must be positive");

        let mut rng = thread_rng();

        #[rustfmt::skip]
        let cells: Vec<Vec<_>> =
            (0..height).map(|_row| {
                (0..width).map(|_col| {
                    if rng.gen_bool(0.5) {
                        Cell::Dead
                    } else {
                        Cell::Alive
                    }
                }).collect()
            }).collect();

        Ok(Self { cells })
    }

    pub fn size(&self) -> (usize, usize) {
        let width = self.cells[0].len();
        let height = self.cells.len();
        (width, height)
    }

    pub fn tick(&mut self) {
        let (width, height) = self.size();

        #[rustfmt::skip]
        let next_cells: Vec<Vec<_>> =
            (0..height).map(|row|
                (0..width).map(|col|
                    self.tick_cell(row, col)
                ).collect()
            ).collect();

        self.cells = next_cells;
    }

    fn tick_cell(&self, row: usize, col: usize) -> Cell {
        let n_alive = self.count_alive_neighbors(row, col);
        match self.cells[row][col] {
            Cell::Dead => {
                if n_alive == 3 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            }
            Cell::Alive => {
                if n_alive == 2 || n_alive == 3 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            }
        }
    }

    fn count_alive_neighbors(&self, row: usize, col: usize) -> usize {
        let (width, height) = self.size();
        let mut n_alive = 0;

        for dy in [height - 1, 0, 1] {
            for dx in [width - 1, 0, 1] {
                if (dy, dx) != (0, 0) {
                    let nrow = (row + dy) % height;
                    let ncol = (col + dx) % width;
                    if self.cells[nrow][ncol] == Cell::Alive {
                        n_alive += 1;
                    }
                }
            }
        }

        n_alive
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Cell {
        self.cells[row][col]
    }
}
