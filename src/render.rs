use std::io::{stdout, Write};

use bracket_lib::terminal::{to_cp437, BLACK, GRAY, RED, RGB};

use crate::{state::State, world::Cell};

pub trait Render {
    fn render(&self, ctx: &mut bracket_lib::terminal::BTerm);
}

fn cell_style(cell: &Cell) -> (RGB, RGB, char) {
    match cell {
        Cell::Ground => (RGB::named(GRAY), RGB::named(GRAY), ' '),
        Cell::Wall => (RGB::named(BLACK), RGB::named(GRAY), '#'),
    }
}

impl Render for State {
    fn render(&self, ctx: &mut bracket_lib::terminal::BTerm) {
        ctx.set_active_console(0);
        ctx.cls();
        for (x, y, cell) in self.world.enumerate_cells() {
            let (fg, bg, sym) = cell_style(cell);
            ctx.set(x, y, bg, fg, to_cp437(sym));
        }
        ctx.set_active_console(1);
        for c in self.iter_creatures() {
            let (x, y) = c.get_pos();
            let (_, bg, _) = cell_style(self.world.get_cell(x, y).unwrap());
            ctx.set(x, y, bg, RGB::named(RED), to_cp437('☺'));
        }
        stdout().flush().unwrap();
    }
}
