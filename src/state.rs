use crate::{creatures::Creature, world::World};

pub struct State {
    pub world: World,
    pub creatures: Vec<Box<dyn Creature>>,
}

impl State {
    pub fn from_world(world: World) -> Self {
        State {
            world,
            creatures: vec![],
        }
    }

    pub fn iter_creatures(&self) -> impl Iterator<Item = &Box<dyn Creature>> {
        self.creatures.iter()
    }

    pub fn add_creature(&mut self, c: Box<dyn Creature>) {
        self.creatures.push(c);
    }
}
