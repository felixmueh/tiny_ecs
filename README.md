# tiny_ecs

An entity-component system written from scratch in January 2023, as an
experiment to learn the type system of Rust. It is a learning project, not a
library to depend on.

## How it works

- **Generational indices.** Entities are `GenIndex { idx, gen }` handles into
  `GenVec` storages. Removing an entity frees its slot and puts it on a free
  list with the next generation, so a stale handle no longer resolves.
- **Type-erased component storage.** Each component type has its own
  `GenVec<Option<T>>`, kept in a `HashMap<TypeId, Box<dyn AnyComponent>>` and
  downcast through `Any` on access. All storages share the same slots and
  generations.
- **Typed tuple queries.** The `Query` trait has a generic associated type
  `Item<'w>`. It is implemented for `&T`, `&mut T`, `Option<&T>` and
  `Option<&mut T>`, and through `macro_rules!` for tuples of two to eight of
  them, zipped with `itertools::multizip`. The `Set` trait does the same for
  writing components.
- **Runtime aliasing check.** A mutable tuple query takes several `&mut` into
  the same `Ecs` through a raw pointer. A query or `Set` that names one
  component type twice, such as `(&mut HP, &HP)`, panics before that happens.
- **Derive macro.** `#[derive(Component)]` from the `component-derive`
  proc-macro crate implements the marker trait `Component`, which a type needs
  before it can be stored.
- The entity's own `Index` is stored as a component, so a query can ask for
  `&Index`.

## Usage

```toml
[dependencies]
tiny_ecs = { git = "https://github.com/felixmueh/tiny_ecs" }
```

```rust
use tiny_ecs::component_derive::Component;
use tiny_ecs::{Component, Ecs, Index};

#[derive(Debug, Component)]
struct Position {
    x: i64,
    y: i64,
}

#[derive(Debug, Component)]
struct Velocity {
    x: i64,
    y: i64,
}

fn main() -> Result<(), tiny_ecs::error::Error> {
    let mut ecs = Ecs::new();

    // Component storages are created on first use.
    let moving = ecs.new_entity_create_components((
        Position { x: 0, y: 0 },
        Velocity { x: 1, y: 2 },
    ))?;
    let fixed = ecs.new_entity_create_components(Position { x: 5, y: 5 })?;

    // Entities without a Velocity are skipped.
    for (pos, vel) in ecs.iter_mut::<(&mut Position, &Velocity)>().unwrap() {
        pos.x += vel.x;
        pos.y += vel.y;
    }
    let pos = ecs.get_entity::<&Position>(moving).unwrap();
    assert_eq!((pos.x, pos.y), (1, 2));

    // Optional components yield &Option<T>.
    for (idx, vel) in ecs.iter::<(&Index, Option<&Velocity>)>().unwrap() {
        println!("{idx}: {vel:?}");
    }

    // The freed slot is reused with the next generation.
    ecs.remove_entity(fixed)?;
    let reused = ecs.new_empty_entity();
    assert_eq!((reused.idx, reused.gen), (fixed.idx, fixed.gen + 1));
    assert!(ecs.get_entity::<&Position>(fixed).is_none());
    Ok(())
}
```

## Tests

```sh
cargo test
```

This runs the 20 unit tests in `src/ecs.rs` and `src/query.rs`.

## Status

Written in two weeks and not developed since. It needs Rust 1.65 or newer
(generic associated types, `let`-`else`).

Every query walks all entity slots through boxed iterators. Error handling is
incomplete, and archetypes, systems and scheduling are missing.

## License

MIT. See `LICENSE`.
