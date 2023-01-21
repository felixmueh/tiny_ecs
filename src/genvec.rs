use crate::error::Error;
use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct GenEntry<T> {
    pub val: Option<T>,
    pub gen: u64,
}

impl<T> GenEntry<T> {
    pub fn new(val: T) -> Self {
        GenEntry {
            val: Some(val),
            gen: 0,
        }
    }
}

impl<T: Default> Default for GenEntry<T> {
    fn default() -> Self {
        Self {
            val: Some(Default::default()),
            gen: Default::default(),
        }
    }
}

#[derive(Debug)]
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
        self.0.push(GenEntry {
            val: Some(val),
            gen,
        });
    }

    pub fn push_hole(&mut self, gen: u64) {
        self.0.push(GenEntry { val: None, gen })
    }

    pub fn get(&self, idx: GenIndex) -> Result<&T> {
        let GenEntry { val, gen } = self.0.get(idx.idx).ok_or(format!(
            "Index out of bounds: '{}' for lentgh '{}'",
            idx.idx,
            self.0.len()
        ))?;
        if *gen == idx.gen {
            Ok(val.as_ref().ok_or("Trying to access a freed slot.")?)
        } else {
            Err(Error::Generic(format!(
                "Invalid Generation expected: {} but got {}",
                gen, idx.gen
            )))
        }
    }

    pub fn get_mut(&mut self, idx: GenIndex) -> Result<&mut T> {
        let len = self.0.len();
        let GenEntry { val, gen } = self.0.get_mut(idx.idx).ok_or(format!(
            "Index out of bounds: '{}' for lentgh '{}'",
            idx.idx, len
        ))?;
        if *gen == idx.gen {
            Ok(val.as_mut().ok_or("Trying to access a freed slot.")?)
        } else {
            Err(Error::Generic(format!(
                "Invalid Generation expected: {} but got {}",
                gen, idx.gen
            )))
        }
    }

    /// Set only works if there is a valid entry at idx already.
    pub fn set(&mut self, idx: GenIndex, value: T) -> Result<()> {
        *self.get_mut(idx)? = value;
        Ok(())
    }

    /// Repopulate with val if slot is free.
    pub fn repop(&mut self, idx: GenIndex, value: T) -> Result<()> {
        let Some(GenEntry { val, gen }) = self.0.get_mut(idx.idx) else {
            return Err(Error::Generic(format!("Index out of bounds: '{}'.", idx)));
        };
        if val.is_some() {
            return Err(Error::Generic(format!(
                "Value is already populated at index: '{}'.",
                idx
            )));
        }
        *val = Some(value);
        *gen += 1;
        Ok(())
    }

    pub fn free(&mut self, idx: GenIndex) -> Result<()> {
        let Some(GenEntry { val, gen }) = self.0.get_mut(idx.idx) else {
            return Err(Error::Generic(format!("Invlid index: '{}'.", idx)));
        };
        if *gen != idx.gen {
            return Err(Error::Generic(format!(
                "Invlid index generation: '{}' expected: '{}'",
                idx, idx.gen
            )));
        }
        if val.is_none() {
            return Err(Error::Generic("Entry already freed".to_string()));
        }
        *val = None;
        Ok(())
    }

    // Iterates over all entries ignoring holes.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().filter_map(|v| v.val.as_ref())
    }

    // Iterates over all entries ignoring holes.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.0.iter_mut().filter_map(|v| v.val.as_mut())
    }
}

impl<T: Default> GenVec<T> {
    /// If the slot is free repopulate it with the default value.
    pub fn repop_default(&mut self, idx: GenIndex) -> Result<()> {
        self.repop(idx, Default::default())
    }
}
