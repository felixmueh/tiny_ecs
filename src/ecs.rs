#![allow(dead_code)]

use crate::{
    genvec::{GenIndex, GenVec},
    prelude::*,
    query::{Query, Set},
};

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub type Index = GenIndex;
pub type Storage<T> = GenVec<Option<T>>;

pub trait Component {}

// Any components are used dynamically store the storage types and still implement
// some methods on them.
#[allow(clippy::len_without_is_empty)]
pub trait AnyComponent: Any {
    // append new entity
    fn push_new(&mut self, gen: u64);
    // repopulate freed entitiy
    fn repop(&mut self, idx: Index) -> Result<()>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn len(&self) -> usize;
    fn indices<'a>(&'a self) -> Box<dyn Iterator<Item = GenIndex> + 'a>;
    fn is_used(&self, idx: usize) -> bool;
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

    fn is_used(&self, idx: usize) -> bool {
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

    fn is_used(&self, idx: usize) -> bool {
        self.as_ref().is_used(idx)
    }

    fn remove_entity(&mut self, idx: Index) -> Result<()> {
        self.as_mut().remove_entity(idx)
    }

    fn repop(&mut self, idx: Index) -> Result<()> {
        self.as_mut().repop(idx)
    }
}

// Implements sensible component like iteration over the storage types.
pub trait ComponentStorage<T> {
    fn iter_valid<'a>(&'a self) -> Box<dyn Iterator<Item = &T> + 'a>;
}

pub trait ComponentStorageMut<T> {
    fn iter_valid_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut T> + 'a>;
}

impl<T> ComponentStorage<T> for Storage<T> {
    fn iter_valid<'a>(&'a self) -> Box<dyn Iterator<Item = &T> + 'a> {
        Box::new(self.iter().filter_map(|v| v.as_ref()))
    }
}

impl<T> ComponentStorageMut<T> for Storage<T> {
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
    components: HashMap<TypeId, Box<dyn AnyComponent>>,
    // Stores Generational Indices of the next free variable (with updated generation.)
    free_list: Vec<Index>,
}

impl Default for Ecs {
    fn default() -> Self {
        Self::new()
    }
}

impl Ecs {
    pub fn new() -> Self {
        Ecs {
            components: HashMap::new(),
            free_list: vec![],
        }
    }

    pub fn add_component<T: 'static>(&mut self) {
        let mut c = Storage::<T>::default();
        if !self.components.is_empty() {
            // Fill the component with the correct generations and slot holes of the other components
            let e1 = self.components.values().next().unwrap().as_ref();
            c.0.reserve(e1.len());
            for idx in e1.indices() {
                if e1.is_used(idx.idx) {
                    c.push(None, idx.gen);
                } else {
                    c.push_hole(idx.gen);
                }
            }
        }
        self.components.insert(TypeId::of::<T>(), Box::new(c));
    }

    pub fn get_component<T: 'static>(&self) -> Option<&Storage<T>> {
        Some(
            self.components
                .get(&TypeId::of::<T>())?
                .as_any()
                .downcast_ref()
                .unwrap(),
        )
    }

    pub fn get_component_mut<T: 'static>(&mut self) -> Option<&mut Storage<T>> {
        Some(
            self.components
                .get_mut(&TypeId::of::<T>())?
                .as_any_mut()
                .downcast_mut()
                .unwrap(),
        )
    }

    /// Adds a new entity without any components
    pub fn add_entity(&mut self) -> Index {
        assert!(
            !self.components.is_empty(),
            "Can't add entity if there is no component."
        );
        if let Some(next_free) = self.free_list.pop() {
            for v in self.components.values_mut() {
                v.repop(next_free).unwrap();
            }
            next_free
        } else {
            for s in self.components.values_mut() {
                s.push_new(0);
            }
            GenIndex {
                idx: self.components.values().next().unwrap().len() - 1,
                gen: 0,
            }
        }
    }

    pub fn set_entity<T>(&mut self, idx: Index, value: T) -> Result<()>
    where
        T: Set<T> + 'static,
    {
        <T>::set_or_create(self, idx, value)
    }

    pub fn get_entity_mut<T>(&mut self, idx: Index) -> Option<<T as Query<T>>::Item<'_>>
    where
        T: Query<T> + 'static,
    {
        <T>::get_mut(self, idx)
    }

    pub fn remove_entity(&mut self, idx: Index) -> Result<()> {
        for c in self.components.values_mut() {
            c.remove_entity(idx)?;
        }
        self.free_list.push(idx.next());
        Ok(())
    }

    fn iter_mut<'w, T>(
        &'w mut self,
    ) -> Option<Box<dyn Iterator<Item = <T as Query<T>>::Item<'w>> + 'w>>
    where
        T: Query<T> + 'static,
    {
        Some(Box::new(T::query_mut(self)?.flatten()))
    }
}

#[cfg(test)]
mod tests {
    use component_derive::Component;

    use super::*;

    #[test]
    fn test_set_entity() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i32>();
        let idx = ecs.add_entity();
        ecs.set_entity(idx, (42, true)).unwrap();
        assert_eq!(
            ecs.get_entity_mut::<(&i32, &bool)>(idx).unwrap(),
            (&42, &true)
        );
    }

    #[test]
    fn better_ergonomics() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        ecs.add_component::<bool>();
        let i1 = ecs.add_entity();
        let i2 = ecs.add_entity();
        ecs.set_entity(i1, true).unwrap();
        ecs.set_entity(i1, 5_i64).unwrap();
        ecs.set_entity(i2, false).unwrap();
        ecs.set_entity(i2, 5_i64).unwrap();

        for (p, b) in ecs.iter_mut::<(&mut i64, &bool)>().unwrap() {
            if *b {
                *p += 1
            }
        }

        let mut iter = ecs.iter_mut::<&i64>().unwrap();
        assert_eq!(*(iter.next().unwrap()), 6_i64);
        assert_eq!(*(iter.next().unwrap()), 5_i64);
    }

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
        assert!(ecs
            .get_component::<i64>()
            .unwrap()
            .get(idx)
            .unwrap()
            .is_none());
        ecs.set_entity(idx, 5_i64).unwrap();
        assert_eq!(
            ecs.get_component::<i64>()
                .unwrap()
                .get(idx)
                .unwrap()
                .unwrap(),
            5_i64
        );
    }

    #[test]
    fn remove_entity() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        let idx = ecs.add_entity();
        ecs.set_entity(idx, 5_i64).unwrap();
        assert!(ecs
            .get_component::<i64>()
            .unwrap()
            .get(idx)
            .unwrap()
            .is_some());
        ecs.remove_entity(idx).unwrap();
        assert!(ecs.get_component::<i64>().unwrap().get(idx).is_none());
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
            ecs.get_component::<i64>()
                .unwrap()
                .iter()
                .nth(1)
                .unwrap()
                .unwrap(),
            5_i64
        );
        assert_eq!(
            *ecs.get_component::<i64>()
                .unwrap()
                .iter_valid()
                .next()
                .unwrap(),
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
            .unwrap()
            .iter_valid_mut()
            .next()
            .unwrap() = 7;
        assert_eq!(
            *ecs.get_component::<i64>()
                .unwrap()
                .iter_valid()
                .next()
                .unwrap(),
            7_i64
        );
    }

    #[test]
    fn access_one_entity() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        let i = ecs.add_entity();
        let v = ecs.get_entity_mut::<Option<&mut i64>>(i).unwrap();
        v.replace(5_i64);
        assert_eq!(
            ecs.get_component::<i64>().unwrap().get(i).unwrap().unwrap(),
            5_i64
        );
    }

    #[test]
    fn access_one_entity_multiple_fields() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        ecs.add_component::<bool>();
        let i = ecs.add_entity();
        let (v, b) = ecs
            .get_entity_mut::<(Option<&mut i64>, Option<&mut bool>)>(i)
            .unwrap();
        v.replace(5_i64);
        b.replace(true);
        assert_eq!(
            ecs.get_component::<i64>().unwrap().get(i).unwrap().unwrap(),
            5_i64
        );
        assert!(ecs
            .get_component::<bool>()
            .unwrap()
            .get(i)
            .unwrap()
            .unwrap(),);
    }

    #[test]
    fn complicated_example() {
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

        let mut ecs = Ecs::new();
        ecs.add_component::<Position>();
        ecs.add_component::<Velocity>();

        let _ = ecs.add_entity();
        let i2 = ecs.add_entity();
        let i3 = ecs.add_entity();

        for e in ecs.get_component_mut::<Position>().unwrap().iter_mut() {
            *e = Some(Position { x: 0, y: 0 });
        }

        ecs.set_entity(i2, Velocity { x: 1, y: 1 }).unwrap();
        ecs.set_entity(i3, Velocity { x: -1, y: 1 }).unwrap();

        for (p, v) in ecs.iter_mut::<(&mut Position, &Velocity)>().unwrap() {
            p.x += v.x;
            p.y += v.y;
        }

        for p in ecs.iter_mut::<&Position>().unwrap() {
            dbg!(p);
        }
    }
}
