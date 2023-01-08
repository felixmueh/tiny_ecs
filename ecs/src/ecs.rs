#![allow(dead_code)]

use crate::{
    error::Error,
    genvec::{GenEntry, GenIndex, GenVec},
    prelude::*,
};

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

trait AnyComponent: Any {
    fn push_new(&mut self, gen: u64);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn len(&self) -> usize;
    fn indices<'a>(&'a self) -> Box<dyn Iterator<Item = GenIndex> + 'a>;
}

impl<T: 'static> AnyComponent for Storage<T> {
    fn push_new(&mut self, gen: u64) {
        self.0.push(GenEntry { gen, val: None });
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
        return self.as_ref().indices();
    }
}

type Index = GenIndex;
type Storage<T> = GenVec<Option<T>>;

// Holds a entity vector for every component type.
// Entity vectors are accessed with generational indices.
// The generation of a single entity is equal in all component vectors.
struct Ecs {
    entities: HashMap<TypeId, Box<dyn AnyComponent>>,
    free_list: Vec<Index>,
    is_free: GenVec<bool>,
}

impl Ecs {
    fn new() -> Self {
        Ecs {
            entities: HashMap::new(),
            free_list: vec![],
            is_free: Default::default(),
        }
    }

    fn add_component<T: 'static>(&mut self) {
        let mut c = Storage::<T>::default();
        if !self.entities.is_empty() {
            // Fill the component with the correct generations of the other components
            let e1 = self.entities.values().next().unwrap().as_ref();
            c.0.reserve(e1.len());
            for idx in e1.indices() {
                c.push_new(idx.gen);
            }
        }
        self.entities.insert(TypeId::of::<T>(), Box::new(c));
    }

    fn get_component<T: 'static>(&self) -> &Storage<T> {
        self.entities
            .get(&TypeId::of::<T>())
            .unwrap()
            .as_any()
            .downcast_ref()
            .unwrap()
    }

    fn get_component_mut<T: 'static>(&mut self) -> &mut Storage<T> {
        self.entities
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .as_any_mut()
            .downcast_mut()
            .unwrap()
    }

    fn add_entity(&mut self) -> Index {
        assert!(
            !self.entities.is_empty(),
            "Can't add entity if there is no component."
        );
        for (_, s) in self.entities.iter_mut() {
            s.push_new(0);
        }
        self.is_free.push(false, 0);
        GenIndex {
            idx: self.entities.iter().next().unwrap().1.len() - 1,
            gen: 0,
        }
    }

    fn check_valid_index(&self, idx: Index) -> Result<()> {
        if idx.idx >= self.is_free.0.len() {
            return Err(Error::Generic(format!(
                "Index out of bounds: '{}'.",
                idx.idx
            )));
        };
        let Some(&free) = self.is_free.get(idx) else {
            return Err(Error::Generic(format!("Index '{}' has invalid generation", idx)));
        };
        if free {
            return Err(Error::Generic(format!("No entity at index '{}'", idx.idx)));
        }
        Ok(())
    }

    fn set_entity<T: 'static>(&mut self, idx: Index, value: T) -> Result<()> {
        self.check_valid_index(idx)?;
        self.get_component_mut::<T>().set(idx, Some(value))
    }

    fn get_entity<T: 'static>(&self, idx: Index) -> Option<&Option<T>> {
        if self.check_valid_index(idx).is_err() {
            return None;
        }
        self.get_component::<T>().get(idx)
    }

    fn get_entity_mut<T: 'static>(&mut self, idx: Index) -> Option<&mut Option<T>> {
        if self.check_valid_index(idx).is_err() {
            return None;
        }
        self.get_component_mut::<T>().get_mut(idx)
    }

    fn remove_entity(&mut self, idx: Index) -> Result<()> {
        self.check_valid_index(idx)?;
        self.free_list.push(idx.next());
        self.is_free.set(idx, true)
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
        ecs.set_entity(idx, 5_i64).unwrap();
        assert_eq!(ecs.get_component::<i64>().get(idx).unwrap().unwrap(), 5_i64);
    }

    #[test]
    fn remove_entity() {
        let mut ecs = Ecs::new();
        ecs.add_component::<i64>();
        let idx = ecs.add_entity();
        ecs.set_entity(idx, 5_i64).unwrap();
        ecs.remove_entity(idx).unwrap();
        assert!(ecs.get_component::<i64>().get(idx).is_none());
    }
}
