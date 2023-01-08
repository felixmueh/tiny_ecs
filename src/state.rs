use crate::world::World;
use ecs::Ecs;

pub struct State {
    pub world: World,
    pub ecs: Ecs,
}

pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl State {
    pub fn from_world(world: World) -> Self {
        let mut ecs = Ecs::new();
        ecs.add_component::<Position>();
        State { world, ecs }
    }
}
