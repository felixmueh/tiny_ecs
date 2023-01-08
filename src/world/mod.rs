mod cell;

pub use self::cell::Cell;

const DEFAULT_SX: usize = 512;
const DEFAULT_SY: usize = 512;

pub struct World {
    sx: usize,
    sy: usize,

    cells: Vec<Vec<Cell>>,
}

impl World {
    pub fn width(&self) -> usize {
        self.sx
    }

    pub fn height(&self) -> usize {
        self.sy
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<&Cell> {
        self.cells.get(y)?.get(x)
    }

    pub fn enumerate_cells(&self) -> impl Iterator<Item = (usize, usize, &Cell)> {
        self.cells
            .iter()
            .enumerate()
            .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, c)| (x, y, c)))
    }
}

impl World {
    pub fn new(sx: usize, sy: usize) -> Self {
        let mut cells = vec![vec![Cell::Wall; sx]; sy];
        if sx >= 2 || sy >= 2 {
            for y in 1..(sy - 1) {
                for x in 1..(sx - 1) {
                    cells[y][x] = Cell::Ground;
                }
            }
        }

        World { sx, sy, cells }
    }

    pub fn update(&mut self) {}
}

impl Default for World {
    fn default() -> Self {
        Self::new(DEFAULT_SX, DEFAULT_SY)
    }
}
