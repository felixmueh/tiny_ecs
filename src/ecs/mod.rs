#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use crate::{error::Error, prelude::*};

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

trait AnyComponent: Any {
    fn push_new(&mut self);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn len(&self) -> usize;
}

impl<T: 'static> AnyComponent for Storage<T> {
    fn push_new(&mut self) {
        self.0.push(GenEntry { val: None, gen: 0 });
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
}

impl AnyComponent for Box<dyn AnyComponent> {
    fn push_new(&mut self) {
        self.as_mut().push_new();
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
}

struct GenIndex {
    idx: usize,
    gen: u64,
}

struct GenEntry<T> {
    val: T,
    gen: u64,
}

struct GenVec<T>(Vec<T>);

impl<T> Default for GenVec<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> GenVec<GenEntry<T>> {
    fn new() -> Self {
        Self(vec![])
    }

    fn get(&self, idx: &GenIndex) -> Option<&T> {
        let GenEntry { val, gen } = self.0.get(idx.idx)?;
        if *gen == idx.gen {
            Some(val)
        } else {
            None
        }
    }

    fn get_mut(&mut self, idx: &GenIndex) -> Option<&mut T> {
        let GenEntry { val, gen } = self.0.get_mut(idx.idx)?;
        if *gen == idx.gen {
            Some(val)
        } else {
            None
        }
    }

    fn set(&mut self, idx: &GenIndex, value: T) -> Result<()> {
        if let Some(val) = self.get_mut(idx) {
            *val = value;
            Ok(())
        } else {
            Err(Error::Generic("Invalid generation.".to_string()))
        }
    }
}

type Index = GenIndex;
type Storage<T> = GenVec<GenEntry<Option<T>>>;

struct Ecs {
    entities: HashMap<TypeId, Box<dyn AnyComponent>>,
    free_list: Vec<usize>,
    is_free: Vec<bool>,
}

impl Ecs {
    fn new() -> Self {
        Ecs {
            entities: HashMap::new(),
            free_list: vec![],
            is_free: vec![],
        }
    }

    fn add_component<T: 'static>(&mut self) {
        self.entities
            .insert(TypeId::of::<T>(), Box::<Storage<T>>::default());
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
        for (t_id, s) in self.entities.iter_mut() {
            s.push_new();
        }
        self.is_free.push(false);
        GenIndex {
            idx: self.entities.iter().next().unwrap().1.len() - 1,
            gen: 0,
        }
    }

    fn set_entity<T: 'static>(&mut self, idx: &Index, value: T) -> Result<()> {
        // TODO: Check everywhree if is_free is false before accessing
        self.get_component_mut::<T>().set(idx, Some(value))
    }

    fn get_entity<T: 'static>(&self, idx: &Index) -> Option<&Option<T>> {
        self.get_component::<T>().get(idx)
    }

    fn get_entity_mut<T: 'static>(&mut self, idx: &Index) -> Option<&mut Option<T>> {
        self.get_component_mut::<T>().get_mut(idx)
    }

    fn remove_entity(&mut self, idx: Index) {
        todo!()
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
        ecs.set_entity(&idx, 5_i64).unwrap();
        assert_eq!(
            ecs.get_component::<i64>().get(&idx).unwrap().unwrap(),
            5_i64
        );
    }
}
