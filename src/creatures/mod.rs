pub mod dwarf;

pub trait Creature {
    fn get_pos(&self) -> (usize, usize);
}
