//! my own entity component system

pub mod entity;
pub mod world;

pub use entity::Entity;

use crate::ecs::entity::EntityBuilder;
use crate::ecs::world::World;
use std::fmt::Debug;
use std::iter::{Enumerate, FilterMap};
use std::net::Shutdown::Write;
use std::ops::Deref;
use std::slice::Iter;

/// An interface for component storages. See `VecStorage` for example implementation
pub trait ComponentStorage<'d, C: 'd + Component> {
    type Reader: ReadAccess<'d, C>;
    type Writer: WriteAccess<C>;

    /// returns slice of components, indexed by entity
    fn read(&self) -> Self::Reader;

    /// writes new component value for entity
    fn write<T>(&mut self) -> Self::Writer;
}

/// An interface for Component. Doesn't actually do anything yet, other than make sure our components are sized, and shareable across threads
pub trait Component: Sized + Send + Sync {}

/// An interface that describes read access to a Component
pub trait ReadAccess<'d, C: 'd + Component>: IntoIterator<Item = (Entity, &'d C)> {
    fn fetch(&self, entity: Entity) -> Option<&C>;
}

/// An interface that describes write access to a Component
pub trait WriteAccess<C: Component> {
    /// sets value of Component for Entity
    fn set(&mut self, entity: Entity, value: C);

    /// unsets value of Component for Entity
    fn unset(&mut self, entity: Entity);

    /// clears this Component storage, unsetting the value for each Entity
    fn clear(&mut self);
}

// Storage types

/// Sparse vector storage. Possible the fastest type in terms of read access.
/// Write access can be horrible slow (might need to reallocate large vectors), and has bad
/// memory usage.
#[derive(Debug)]
pub struct VecStorage<C: Component + Debug> {
    data: Vec<Option<C>>,
}

impl<C: Component + Debug> Default for VecStorage<C> {
    fn default() -> Self {
        VecStorage { data: Vec::new() }
    }
}

impl<C: Component + Debug> VecStorage<C> {
    pub fn new() -> VecStorage<C> {
        VecStorage::default()
    }
}

/// Read accessor for VecStorage
pub struct VecReader<'v, C> {
    data: &'v Vec<Option<C>>,
}

impl<'v, C> VecReader<'v, C> {
    pub fn new(data: &'v Vec<Option<C>>) -> VecReader<'v, C> {
        VecReader { data }
    }
}

// this here is the problem
impl<'v, C: Component> IntoIterator for VecReader<'v, C> {
    type Item = (Entity, &'v C);
    type IntoIter = FilterMap<
        Enumerate<Iter<'v, Option<C>>>,
        fn((usize, &'v Option<C>)) -> Option<(Entity, &'v C)>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(index, maybe_val)| match maybe_val {
                Some(val) => Some((Entity(index), val)),
                None => None,
            })
    }
}

// ReadAccess requires the struct to implement IntoIterator
impl<'v, C: 'v + Component> ReadAccess<'v, C> for VecReader<'v, C> {
    fn fetch(&self, entity: Entity) -> Option<&C> {
        self.data.get(*entity).unwrap_or(&None).as_ref()
    }
}

/// Write access to a VecStorage. Uses mutable borrow so there can only exists one writer at a time.
pub struct VecWriter<'v, C> {
    data: &'v mut Vec<Option<C>>,
}

impl<'v, C: Component> VecWriter<'v, C> {
    pub fn new(data: &'v mut Vec<Option<C>>) -> VecWriter<'v, C> {
        VecWriter { data }
    }
}

impl<'v, C: Component> WriteAccess<C> for VecWriter<'v, C> {
    fn set(&mut self, entity: Entity, value: C) {
        if self.data.capacity() <= *entity {
            self.data.reserve(*entity - self.data.capacity() + 1);
        }

        if self.data.len() <= *entity {
            for _ in 1..*entity - self.data.len() {
                self.data.push(None);
            }
            self.data.push(Some(value));
        } else {
            self.data[*entity] = Some(value);
        }
    }

    fn unset(&mut self, entity: Entity) {
        unimplemented!()
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

// finally, we can implement CompontentStorage for VecStorage using the reader and writer we
// implemented above
impl<'v, C: 'v + Component + Debug> ComponentStorage<'v, C> for VecStorage<C> {
    type Reader = VecReader<'v, C>;
    type Writer = VecWriter<'v, C>;

    fn read(&self) -> Self::Reader {
        VecReader::new(&self.data)
    }

    fn write<T>(&mut self) -> Self::Writer {
        VecWriter::new(&mut self.data)
    }
}
