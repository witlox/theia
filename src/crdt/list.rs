use core::fmt;
use core::iter::FromIterator;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use crate::crdt::serde_ext::SerDe;
use crate::crdt::{Identifier, CmRDT, VectorClock, Version, VersionRange};
use crate::crdt::version::OrderedVersion;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct List<T: SerDe, A: Ord> {
    #[serde(with="crate::crdt::serde_ext::btreemap_to_vec")]
    sequence: BTreeMap<Identifier<OrderedVersion<A>>, T>,
    clock: VectorClock<A>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation<T, A: Ord> {
    Insert {
        id: Identifier<OrderedVersion<A>>,
        value: T,
    },
    Delete {
        id: Identifier<OrderedVersion<A>>,
        version: Version<A>,
    },
}

impl<T, A: Ord + Clone + Eq> Operation<T, A> {
    pub fn id(&self) -> &Identifier<OrderedVersion<A>> {
        match self {
            Operation::Insert { id, .. } | Operation::Delete { id, .. } => id,
        }
    }

    pub fn version(&self) -> Version<A> {
        match self {
            Operation::Insert { id, .. } => id.value().clone().into(),
            Operation::Delete { version: dot, .. } => dot.clone(),
        }
    }
}

impl<T: SerDe, A: Ord> Default for List<T, A> {
    fn default() -> Self {
        Self {
            sequence: Default::default(),
            clock: Default::default(),
        }
    }
}

impl<T: SerDe, A: Ord + Clone> List<T, A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_index(&self, mut index: usize, element: T, actor: A) -> Operation<T, A> {
        index = index.min(self.sequence.len());
        let (prev, next) = match index.checked_sub(1) {
            Some(indices_to_drop) => {
                let mut indices = self.sequence.keys().skip(indices_to_drop);
                (indices.next(), indices.next())
            }
            None => {
                let mut indices = self.sequence.keys();
                (None, indices.next())
            }
        };

        let version = self.clock.increment(actor);
        let id = Identifier::between(prev, next, version.into());
        Operation::Insert { id, value: element }
    }

    pub fn append(&self, element: T, actor: A) -> Operation<T, A> {
        let index = self.sequence.len();
        self.insert_index(index, element, actor)
    }

    pub fn delete_index(&self, index: usize, actor: A) -> Option<Operation<T, A>> {
        self.sequence.keys().nth(index).cloned().map(|id| {
            let version = self.clock.increment(actor);
            Operation::Delete { id, version }
        })
    }

    pub fn len(&self) -> usize {
        self.sequence.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sequence.is_empty()
    }

    /// Read the List into a container of your choice
    ///
    /// ```rust
    /// use libtheia::crdt::{List, CmRDT};
    ///
    /// let mut list = List::new();
    /// list.apply(list.append('a', 'A'));
    /// list.apply(list.append('b', 'A'));
    /// list.apply(list.append('c', 'A'));
    /// assert_eq!(list.read::<String>(), "abc");
    /// ```
    pub fn read<'a, C: FromIterator<&'a T>>(&'a self) -> C {
        self.sequence.values().collect()
    }

    /// Read the List into a container of your choice, consuming it.
    ///
    /// ```rust
    /// use libtheia::crdt::{List, CmRDT};
    ///
    /// let mut list = List::new();
    /// list.apply(list.append(1, 'A'));
    /// list.apply(list.append(2, 'A'));
    /// list.apply(list.append(3, 'A'));
    /// assert_eq!(list.read_into::<Vec<_>>(), vec![1, 2, 3]);
    /// ```
    pub fn read_into<C: FromIterator<T>>(self) -> C {
        self.sequence.into_values().collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.sequence.values()
    }

    pub fn iter_entries(&self) -> impl Iterator<Item = (&Identifier<OrderedVersion<A>>, &T)> {
        self.sequence.iter()
    }

    pub fn pos(&self, index: usize) -> Option<&T> {
        self.iter().nth(index)
    }

    pub fn pos_entry(&self, id: &Identifier<OrderedVersion<A>>) -> Option<usize> {
        self.iter_entries()
            .enumerate()
            .find_map(|(ix, (ident, _))| if ident == id { Some(ix) } else { None })
    }

    pub fn get(&self, id: &Identifier<OrderedVersion<A>>) -> Option<&T> {
        self.sequence.get(id)
    }

    pub fn first(&self) -> Option<&T> {
        self.first_entry().map(|(_, val)| val)
    }

    pub fn first_entry(&self) -> Option<(&Identifier<OrderedVersion<A>>, &T)> {
        self.sequence.iter().next()
    }

    pub fn last(&self) -> Option<&T> {
        self.last_entry().map(|(_, val)| val)
    }

    pub fn last_entry(&self) -> Option<(&Identifier<OrderedVersion<A>>, &T)> {
        self.sequence.iter().next_back()
    }

    fn insert(&mut self, id: Identifier<OrderedVersion<A>>, element: T) {
        self.sequence.entry(id).or_insert(element);
    }

    fn delete(&mut self, id: &Identifier<OrderedVersion<A>>) {
        self.sequence.remove(id);
    }
}

impl<T: SerDe, A: Ord + Clone + fmt::Debug> CmRDT for List<T, A> {
    type Operation = Operation<T, A>;
    type Validation = VersionRange<A>;

    fn validate(&self, operation: &Self::Operation) -> Result<(), Self::Validation> {
        self.clock.validate(&operation.version())
    }

    fn apply(&mut self, operation: Self::Operation) {
        let version = operation.version();

        if version.counter <= self.clock.get(&version.actor) {
            return;
        }

        self.clock.apply(version);
        match operation {
            Operation::Insert { id, value: val } => self.insert(id, val),
            Operation::Delete { id, .. } => self.delete(&id),
        }
    }
}

impl<T: SerDe, A: Ord> IntoIterator for List<T, A> {
    type Item = T;

    type IntoIter = std::collections::btree_map::IntoValues<Identifier<OrderedVersion<A>>, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.sequence.into_values()
    }
}