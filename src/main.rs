use bracket_lib::terminal::{main_loop, BTermBuilder, GameState};
use df_clone::creatures::dwarf::Dwarf;
use df_clone::prelude::*;
use df_clone::render::Render;
use df_clone::state::State;
use df_clone::world::World;

const MAX_FPS: usize = 30;

struct LoopState(State);

impl GameState for LoopState {
    fn tick(&mut self, ctx: &mut bracket_lib::terminal::BTerm) {
        self.0.render(ctx);
    }
}

fn main() -> Result<()> {
    let w = World::new(80, 50);
    let mut s = State::from_world(w);
    s.add_creature(Dwarf::spawn(2, 2));

    let context = BTermBuilder::simple80x50()
        .with_font("vga8x16.png", 8u32, 16u32)
        .with_title("DF Clone proj.")
        .with_sparse_console(80, 50, "vga8x16.png")
        .with_fps_cap(MAX_FPS as f32)
        .build()?;

    main_loop(context, LoopState(s))?;
    Ok(())
}
