use super::Creature;

pub struct Dwarf {
    x: usize,
    y: usize,
}

impl Dwarf {
    pub fn spawn(x: usize, y: usize) -> Box<Self> {
        Box::new(Self { x, y })
    }
}

impl Creature for Dwarf {
    fn get_pos(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}
