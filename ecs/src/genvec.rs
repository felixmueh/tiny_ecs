use crate::error::Error;
use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct GenIndex {
    pub idx: usize,
    pub gen: u64,
}

impl GenIndex {
    pub fn next(&self) -> GenIndex {
        GenIndex {
            gen: self.gen + 1,
            ..*self
        }
    }
}

impl std::fmt::Display for GenIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(idx: {}, gen: {})", self.idx, self.gen)
    }
}

pub struct GenEntry<T> {
    pub val: T,
    pub gen: u64,
}

impl<T> GenEntry<T> {
    pub fn new(val: T) -> Self {
        GenEntry { val, gen: 0 }
    }
}

impl<T: Default> Default for GenEntry<T> {
    fn default() -> Self {
        Self {
            val: Default::default(),
            gen: Default::default(),
        }
    }
}

pub struct GenVec<T>(pub Vec<GenEntry<T>>);

impl<T> Default for GenVec<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> GenVec<T> {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn push(&mut self, val: T, gen: u64) {
        self.0.push(GenEntry { val, gen });
    }

    pub fn get(&self, idx: GenIndex) -> Option<&T> {
        let GenEntry { val, gen } = self.0.get(idx.idx)?;
        if *gen == idx.gen {
            Some(val)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, idx: GenIndex) -> Option<&mut T> {
        let GenEntry { val, gen } = self.0.get_mut(idx.idx)?;
        if *gen == idx.gen {
            Some(val)
        } else {
            None
        }
    }

    pub fn set(&mut self, idx: GenIndex, value: T) -> Result<()> {
        if let Some(val) = self.get_mut(idx) {
            *val = value;
            Ok(())
        } else {
            Err(Error::Generic("Invalid generation.".to_string()))
        }
    }
}
