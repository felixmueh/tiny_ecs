#[derive(Debug, Clone)]
pub enum Cell {
    Ground,
    Wall,
}

impl Default for Cell {
    fn default() -> Self {
        Self::Ground
    }
}
