//! Extensions for storing custom data in [`Context`](crate::Context)s.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
};

/// Type map for storing custom data in a [`Context`](crate::Context).
pub struct Extensions {
    map: Option<AnyMap>,
}

impl Extensions {
    /// Create an new empty [`Extensions`] instance.
    pub const fn new() -> Extensions {
        Extensions { map: None }
    }

    /// Check if the [`Extensions`] instance contains data of the given type.
    pub fn has<T: Send + Sync + 'static>(&self) -> bool {
        self.map
            .as_ref()
            .is_some_and(|map| map.contains_key(&TypeId::of::<T>()))
    }

    /// Get a reference to the data of the given type, if it exists.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map
            .as_ref()?
            .get(&TypeId::of::<T>())
            .and_then(|any| any.downcast_ref::<T>())
    }

    /// Get a mutable reference to the data of the given type, if it exists.
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.map
            .as_mut()?
            .get_mut(&TypeId::of::<T>())
            .and_then(|any| any.downcast_mut::<T>())
    }

    /// Insert data of the given type into the [`Extensions`] instance.
    ///
    /// If data of the same type already exists, it will be replaced and returned.
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) -> Option<T> {
        self.map
            .get_or_insert_default()
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|any| any.downcast().ok())
            .map(|boxed| *boxed)
    }

    /// Remove data of the given type from the [`Extensions`] instance.
    ///
    /// If data of the given type exists, it will be removed and returned.
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.map
            .as_mut()?
            .remove(&TypeId::of::<T>())
            .and_then(|any| any.downcast().ok())
            .map(|boxed| *boxed)
    }
}

impl Default for Extensions {
    fn default() -> Self {
        Self::new()
    }
}

type AnyMap = HashMap<TypeId, Box<dyn Any + Send + Sync>, BuildHasherDefault<IdHasher>>;

#[derive(Debug, Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {
        unreachable!("TypeId calls write_u64")
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}
