#![allow(dead_code)]

use crate::{
    genvec::{GenIndex, GenVec},
    prelude::*,
};

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

trait AnyComponent: Any {
    // append new entity
    fn push_new(&mut self, gen: u64);
    // repopulate freed entitiy
    fn repop(&mut self, idx: Index) -> Result<()>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn len(&self) -> usize;
    fn indices<'a>(&'a self) -> Box<dyn Iterator<Item = GenIndex> + 'a>;
    fn is_full(&self, idx: usize) -> bool;
    fn remove_entity(&mut self, idx: Index) -> Result<()>;
}

impl<T: 'static> AnyComponent for Storage<T> {
    fn push_new(&mut self, gen: u64) {
        self.push(None, gen);
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn indices<'a>(&'a self) -> Box<dyn Iterator<Item = GenIndex> + 'a> {
        // Note: caching the indices into a vector to satisfy type system???
        Box::new(
            self.0
                .iter()
                .enumerate()
                .map(|(i, v)| GenIndex { idx: i, gen: v.gen }),
        )
    }

    fn is_full(&self, idx: usize) -> bool {
        self.0[idx].val.is_some()
    }

    fn remove_entity(&mut self, idx: Index) -> Result<()> {
        self.free(idx)
    }

    fn repop(&mut self, idx: Index) -> Result<()> {
        self.repop(idx, None)
    }
}

impl AnyComponent for Box<dyn AnyComponent> {
    fn push_new(&mut self, gen: u64) {
        self.as_mut().push_new(gen);
    }

    fn as_any(&self) -> &dyn Any {
        self.as_ref().as_any()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self.as_mut().as_any_mut()
    }

    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn indices<'a>(&'a self) -> Box<dyn Iterator<Item = GenIndex> + 'a> {
        self.as_ref().indices()
    }

    fn is_full(&self, idx: usize) -> bool {
        self.as_ref().is_full(idx)
    }

    fn remove_entity(&mut self, idx: Index) -> Result<()> {
        self.as_mut().remove_entity(idx)
    }

    fn repop(&mut self, idx: Index) -> Result<()> {
        self.as_mut().repop(idx)
    }
}

type Index = GenIndex;
type Storage<T> = GenVec<Option<T>>;

fn join<'a, A: 'a, B: 'a>(
    c1: &'a Storage<A>,
    c2: &'a Storage<B>,
) -> impl Iterator<Item = (&'a A, &'a B)> {
    c1.iter()
        .zip(c2.iter())
        .filter(|&(a, b)| a.is_some() && b.is_some())
        .map(|(a, b)| (a.as_ref().unwrap(), b.as_ref().unwrap()))
}

fn join_mut<'a, A: 'a, B: 'a>(
    c1: &'a mut Storage<A>,
    c2: &'a mut Storage<B>,
) -> impl Iterator<Item = (&'a mut A, &'a mut B)> {
    c1.iter_mut()
        .zip(c2.iter_mut())
        .filter(|(a, b)| a.is_some() && b.is_some())
        .map(|(a, b)| (a.as_mut().unwrap(), b.as_mut().unwrap()))
}

pub trait Component<T> {
    fn iter_valid<'a>(&'a self) -> Box<dyn Iterator<Item = &T> + 'a>;
}

pub trait ComponentMut<T> {
    fn iter_valid_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut T> + 'a>;
}

impl<T> Component<T> for Storage<T> {
    fn iter_valid<'a>(&'a self) -> Box<dyn Iterator<Item = &T> + 'a> {
        Box::new(self.iter().filter_map(|v| v.as_ref()))
    }
}

impl<T> ComponentMut<T> for Storage<T> {
    fn iter_valid_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut T> + 'a> {
        Box::new(self.iter_mut().filter_map(|v| v.as_mut()))
    }
}

// Holds a entity vector for every component type.
// Entity vectors are accessed with generational indices.
// The generation of a single entity is equal in all component vectors.
// Component vectors might have holes where no entity resides. These are the same in every
// Component and are different of an entity not having a specific component.
pub struct Ecs {
    entities: HashMap<TypeId, Box<dyn AnyComponent>>,
    // Stores Generational Indices of the next free variable (with updated generation.)
    free_list: Vec<Index>,
}

impl Ecs {
    pub fn new() -> Self {
        Ecs {
            entities: HashMap::new(),
            free_list: vec![],
        }
    }

    pub fn add_component<T: 'static>(&mut self) {
        let mut c = Storage::<T>::default();
        if !self.entities.is_empty() {
            // Fill the component with the correct generations and slot holes of the other components
            let e1 = self.entities.values().next().unwrap().as_ref();
            c.0.reserve(e1.len());
            for idx in e1.indices() {
                if e1.is_full(idx.idx) {
                    c.push_hole(idx.gen);
                } else {
                    c.push(None, idx.gen);
                }
            }
        }
        self.entities.insert(TypeId::of::<T>(), Box::new(c));
    }

    pub fn get_components<A: 'static, B: 'static>(&self) -> impl Iterator<Item = (&A, &B)>
    where
        A: 'static,
        B: 'static,
    {
        join(self.get_component::<A>(), self.get_component::<B>())
    }

    pub fn get_components_mut<A: 'static, B: 'static>(
        &mut self,
    ) -> impl Iterator<Item = (&mut A, &mut B)>
    where
        A: 'static,
        B: 'static,
    {
        assert!(TypeId::of::<A>() != TypeId::of::<B>());
        // It is safe to give two mutable references to two different components.
        unsafe {
            join_mut(
                (*(self as *mut Self)).get_component_mut::<A>(),
                self.get_component_mut::<B>(),
            )
        }
    }

    pub fn get_component<T: 'static>(&self) -> &Storage<T> {
        self.entities
            .get(&TypeId::of::<T>())
            .unwrap()
            .as_any()
            .downcast_ref()
            .unwrap()
    }

    pub fn get_component_mut<T: 'static>(&mut self) -> &mut Storage<T> {
        self.entities
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .as_any_mut()
            .downcast_mut()
            .unwrap()
    }

    /// Adds a new entity without any components
    pub fn add_entity(&mut self) -> Index {
        assert!(
            !self.entities.is_empty(),
            "Can't add entity if there is no component."
        );
        if let Some(next_free) = self.free_list.pop() {
            for v in self.entities.values_mut() {
                v.repop(next_free).unwrap();
            }
            next_free
        } else {
            for s in self.entities.values_mut() {
                s.push_new(0);
            }
            GenIndex {
                idx: self.entities.iter().next().unwrap().1.len() - 1,
                gen: 0,
            }
        }
    }

    pub fn set_entity<T: 'static>(&mut self, idx: Index, value: T) -> Result<()> {
        self.get_component_mut::<T>().set(idx, Some(value))
    }

    pub fn get_entity<T: 'static>(&self, idx: Index) -> Option<&Option<T>> {
        self.get_component::<T>().get(idx)
    }

    pub fn get_entity_mut<T: 'static>(&mut self, idx: Index) -> Option<&mut Option<T>> {
        self.get_component_mut::<T>().get_mut(idx)
    }

    pub fn remove_entity(&mut self, idx: Index) -> Result<()> {
        for c in self.entities.values_mut() {
            c.remove_entity(idx)?;
        }
        self.free_list.push(idx.next());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_component() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        ecs.get_component::<i64>();
    }

    #[test]
    fn add_entity() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        let idx = ecs.add_entity();
        assert!(ecs.get_component::<i64>().get(idx).unwrap().is_none());
        ecs.set_entity(idx, 5_i64).unwrap();
        assert_eq!(ecs.get_component::<i64>().get(idx).unwrap().unwrap(), 5_i64);
    }

    #[test]
    fn remove_entity() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        let idx = ecs.add_entity();
        ecs.set_entity(idx, 5_i64).unwrap();
        assert!(ecs.get_component::<i64>().get(idx).unwrap().is_some());
        ecs.remove_entity(idx).unwrap();
        assert!(ecs.get_component::<i64>().get(idx).is_none());
    }

    #[test]
    fn insert_after_remove_entity() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        let idx = ecs.add_entity();
        ecs.remove_entity(idx).unwrap();
        let idx2 = ecs.add_entity();
        assert_eq!(idx.next(), idx2);
    }

    #[test]
    fn iter_component() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        let _ = ecs.add_entity();
        let idx2 = ecs.add_entity();
        ecs.set_entity(idx2, 5_i64).unwrap();
        assert_eq!(
            ecs.get_component::<i64>().iter().nth(1).unwrap().unwrap(),
            5_i64
        );
        assert_eq!(
            *ecs.get_component::<i64>().iter_valid().next().unwrap(),
            5_i64
        );
    }

    #[test]
    fn iter_component_mut() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        let _ = ecs.add_entity();
        let idx2 = ecs.add_entity();
        ecs.set_entity(idx2, 5_i64).unwrap();
        *ecs.get_component_mut::<i64>()
            .iter_valid_mut()
            .next()
            .unwrap() = 7;
        assert_eq!(
            *ecs.get_component::<i64>().iter_valid().next().unwrap(),
            7_i64
        );
    }

    #[test]
    fn iter_join() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        ecs.add_component::<String>();
        let idx = ecs.add_entity();
        ecs.set_entity(idx, 5_i64).unwrap();
        ecs.set_entity(idx, "Test".to_string()).unwrap();
        let (a, b) = join(ecs.get_component::<i64>(), ecs.get_component::<String>())
            .next()
            .unwrap();
        assert_eq!(*a, 5_i64);
        assert_eq!(*b, "Test".to_string());
    }

    #[test]
    fn get_components() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        ecs.add_component::<String>();
        let idx = ecs.add_entity();
        ecs.set_entity(idx, 5_i64).unwrap();
        ecs.set_entity(idx, "Test".to_string()).unwrap();
        let (a, b) = ecs.get_components::<i64, String>().next().unwrap();
        assert_eq!(*a, 5_i64);
        assert_eq!(*b, "Test".to_string());
    }

    #[test]
    fn get_components_mut() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        ecs.add_component::<String>();
        let idx = ecs.add_entity();
        ecs.set_entity(idx, 5_i64).unwrap();
        ecs.set_entity(idx, "Test".to_string()).unwrap();
        let (a, b) = ecs.get_components_mut::<i64, String>().next().unwrap();
        *a = 7;
        *b = "Test2".to_string();
        let (a, b) = ecs.get_components::<i64, String>().next().unwrap();
        assert_eq!(*a, 7_i64);
        assert_eq!(*b, "Test2".to_string());
    }

    #[test]
    fn complicated_example() {
        #[derive(Debug)]
        struct Position {
            x: i64,
            y: i64,
        }

        #[derive(Debug)]
        struct Velocity {
            x: i64,
            y: i64,
        }

        let mut ecs = Ecs::new();
        ecs.add_component::<Position>();
        ecs.add_component::<Velocity>();

        let i1 = ecs.add_entity();
        let i2 = ecs.add_entity();
        let i3 = ecs.add_entity();

        for e in ecs.get_component_mut::<Position>().iter_mut() {
            *e = Some(Position { x: 0, y: 0 });
        }

        ecs.set_entity(i2, Velocity { x: 1, y: 1 }).unwrap();
        ecs.set_entity(i3, Velocity { x: -1, y: 1 }).unwrap();

        for (p, v) in ecs.get_components_mut::<Position, Velocity>() {
            p.x += v.x;
            p.y += v.y;
        }

        for p in ecs.get_component::<Position>().iter_valid() {
            dbg!(p);
        }
        panic!();
    }
}
