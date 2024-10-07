use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Display};
use std::hash::Hash;
use std::mem;

use serde::{Deserialize, Serialize};
use crate::crdt::base::{Add, Read, Remove};
use crate::crdt::{CmRDT, CvRDT, ResetRemove, VectorClock, Version, VersionRange};

pub trait Val<A: Ord>: Clone + Default + ResetRemove<A> + CmRDT {}

impl<A, T> Val<A> for T where A: Ord, T: Clone + Default + ResetRemove<A> + CmRDT {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Map<K: Ord, V: Val<A>, A: Ord + Hash> {
    clock: VectorClock<A>,
    entries: BTreeMap<K, Entry<V, A>>,
    deferred: HashMap<VectorClock<A>, BTreeSet<K>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Entry<V: Val<A>, A: Ord> {
    clock: VectorClock<A>,
    value: V,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation<K: Ord, V: Val<A>, A: Ord> {
    Remove {
        clock: VectorClock<A>,
        key_set: BTreeSet<K>,
    },
    Update {
        version: Version<A>,
        key: K,
        operation: V::Operation,
    },
}

impl<V: Val<A>, A: Ord> Default for Entry<V, A> {
    fn default() -> Self {
        Self {
            clock: VectorClock::default(),
            value: V::default(),
        }
    }
}

impl<K: Ord, V: Val<A>, A: Ord + Hash> Default for Map<K, V, A> {
    fn default() -> Self {
        Self {
            clock: Default::default(),
            entries: Default::default(),
            deferred: Default::default(),
        }
    }
}

impl<K: Ord, V: Val<A>, A: Ord + Hash> ResetRemove<A> for Map<K, V, A> {
    fn reset_remove(&mut self, clock: &VectorClock<A>) {
        self.entries = mem::take(&mut self.entries)
            .into_iter()
            .filter_map(|(key, mut entry)| {
                entry.clock.reset_remove(clock);
                entry.value.reset_remove(clock);
                if entry.clock.is_empty() {
                    None
                } else {
                    Some((key, entry))
                }
            })
            .collect();

        self.deferred = mem::take(&mut self.deferred)
            .into_iter()
            .filter_map(|(mut rm_clock, key)| {
                rm_clock.reset_remove(clock);
                if rm_clock.is_empty() {
                    None
                } else {
                    Some((rm_clock, key))
                }
            })
            .collect();

        self.clock.reset_remove(clock);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CmRDTValidation<V: CmRDT, A> {
    SourceOrder(VersionRange<A>),
    Value(V::Validation),
}

impl<V: CmRDT + Debug, A: Debug> Display for CmRDTValidation<V, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl<V: CmRDT + Debug, A: Debug> std::error::Error for CmRDTValidation<V, A> {}

#[derive(Debug, PartialEq, Eq)]
pub enum CvRDTValidation<K, V: CvRDT, A> {
    DoubleSpentVersion {
        version: Version<A>,
        our_key: K,
        their_key: K,
    },
    Value(V::Validation),
}

impl<K: Debug, V: CvRDT + Debug, A: Debug> Display for CvRDTValidation<K, V, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl<K: Debug, V: CvRDT + Debug, A: Debug> std::error::Error for CvRDTValidation<K, V, A> {}

impl<K: Ord, V: Val<A> + Debug, A: Ord + Hash + Clone + Debug> CmRDT for Map<K, V, A> {
    type Operation = Operation<K, V, A>;
    type Validation = CmRDTValidation<V, A>;

    fn validate_operation(&self, op: &Self::Operation) -> Result<(), Self::Validation> {
        match op {
            Operation::Remove { .. } => Ok(()),
            Operation::Update { version: v, key, operation: op } => {
                self.clock
                    .validate_operation(v)
                    .map_err(CmRDTValidation::SourceOrder)?;
                let entry = self.entries.get(key).cloned().unwrap_or_default();
                entry
                    .clock
                    .validate_operation(v)
                    .map_err(CmRDTValidation::SourceOrder)?;
                entry.value.validate_operation(op).map_err(CmRDTValidation::Value)
            }
        }
    }

    fn apply(&mut self, operation: Self::Operation) {
        match operation {
            Operation::Remove { clock, key_set: key_set } => self.apply_key_set_remove(key_set, clock),
            Operation::Update { version: v, key, operation: op } => {
                if self.clock.get(&v.actor) >= v.counter {
                    return;
                }
                let entry = self.entries.entry(key).or_default();
                entry.clock.apply(v.clone());
                entry.value.apply(op);
                self.clock.apply(v);
                self.apply_deferred();
            }
        }
    }
}

impl<K: Ord + Clone + Debug, V: Val<A> + CvRDT + Debug, A: Ord + Hash + Clone + Debug> CvRDT for Map<K, V, A> {
    type Validation = CvRDTValidation<K, V, A>;

    fn validate_merge(&self, other: &Self) -> Result<(), Self::Validation> {
        for (key, entry) in self.entries.iter() {
            for (other_key, other_entry) in other.entries.iter() {
                for Version { actor, counter } in entry.clock.iterator() {
                    if other_key != key && other_entry.clock.get(actor) == counter {
                        return Err(CvRDTValidation::DoubleSpentVersion {
                            version: Version::new(actor.clone(), counter),
                            our_key: key.clone(),
                            their_key: other_key.clone(),
                        });
                    }
                }

                if key == other_key && entry.clock.concurrent(&other_entry.clock) {
                    entry
                        .value
                        .validate_merge(&other_entry.value)
                        .map_err(CvRDTValidation::Value)?;
                }
            }
        }

        Ok(())
    }

    fn merge(&mut self, other: Self) {
        self.entries = mem::take(&mut self.entries)
            .into_iter()
            .filter_map(|(key, mut entry)| {
                if !other.entries.contains_key(&key) {
                    if other.clock >= entry.clock {
                        None
                    } else {
                        entry.clock.reset_remove(&other.clock);
                        let mut removed_information = other.clock.clone();
                        removed_information.reset_remove(&entry.clock);
                        entry.value.reset_remove(&removed_information);
                        Some((key, entry))
                    }
                } else {
                    Some((key, entry))
                }
            })
            .collect();

        for (key, mut entry) in other.entries {
            if let Some(our_entry) = self.entries.get_mut(&key) {
                let mut common = VectorClock::intersection(&entry.clock, &our_entry.clock);
                common.merge(entry.clock.clone_without(&self.clock));
                common.merge(our_entry.clock.clone_without(&other.clock));
                if common.is_empty() {
                    self.entries.remove(&key).unwrap();
                } else {
                    our_entry.value.merge(entry.value);

                    let mut information_that_was_deleted = entry.clock.clone();
                    information_that_was_deleted.merge(our_entry.clock.clone());
                    information_that_was_deleted.reset_remove(&common);
                    our_entry.value.reset_remove(&information_that_was_deleted);
                    our_entry.clock = common;
                }
            } else {
                if self.clock >= entry.clock {
                } else {
                    entry.clock.reset_remove(&self.clock);

                    let mut information_we_deleted = self.clock.clone();
                    information_we_deleted.reset_remove(&entry.clock);
                    entry.value.reset_remove(&information_we_deleted);
                    self.entries.insert(key, entry);
                }
            }
        }

        for (rm_clock, keys) in other.deferred {
            self.apply_key_set_remove(keys, rm_clock);
        }

        self.clock.merge(other.clock);

        self.apply_deferred();
    }
}

impl<K: Ord, V: Val<A>, A: Ord + Hash + Clone> Map<K, V, A> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_empty(&self) -> Read<bool, A> {
        Read {
            add_clock: self.clock.clone(),
            rm_clock: self.clock.clone(),
            value: self.entries.is_empty(),
        }
    }

    pub fn len(&self) -> Read<usize, A> {
        Read {
            add_clock: self.clock.clone(),
            rm_clock: self.clock.clone(),
            value: self.entries.len(),
        }
    }

    pub fn get(&self, key: &K) -> Read<Option<V>, A> {
        let add_clock = self.clock.clone();
        let entry_opt = self.entries.get(key);
        Read {
            add_clock,
            rm_clock: entry_opt
                .map(|map_entry| map_entry.clock.clone())
                .unwrap_or_default(),
            value: entry_opt.map(|map_entry| map_entry.value.clone()),
        }
    }

    pub fn update<F>(&self, key: impl Into<K>, a: Add<A>, f: F) -> Operation<K, V, A> where F: FnOnce(&V, Add<A>) -> V::Operation
    {
        let key = key.into();
        let version = a.version.clone();
        let operation = match self.entries.get(&key).map(|e| &e.value) {
            Some(data) => f(data, a),
            None => f(&V::default(), a),
        };

        Operation::Update { version, key, operation }
    }

    pub fn remove(&self, key: impl Into<K>, r: Remove<A>) -> Operation<K, V, A> {
        let mut keyset = BTreeSet::new();
        keyset.insert(key.into());
        Operation::Remove {
            clock: r.clock,
            key_set: keyset,
        }
    }

    pub fn read(&self) -> Read<(), A> {
        Read {
            add_clock: self.clock.clone(),
            rm_clock: self.clock.clone(),
            value: (),
        }
    }

    fn apply_deferred(&mut self) {
        let deferred = mem::take(&mut self.deferred);
        for (clock, keys) in deferred {
            self.apply_key_set_remove(keys, clock);
        }
    }

    fn apply_key_set_remove(&mut self, mut keyset: BTreeSet<K>, clock: VectorClock<A>) {
        for key in keyset.iter() {
            if let Some(entry) = self.entries.get_mut(key) {
                entry.clock.reset_remove(&clock);
                if entry.clock.is_empty() {
                    self.entries.remove(key);
                } else {
                    entry.value.reset_remove(&clock);
                }
            }
        }

        match self.clock.partial_cmp(&clock) {
            None | Some(Ordering::Less) => {
                let deferred_set = self.deferred.entry(clock).or_default();
                deferred_set.append(&mut keyset);
            }
            _ => {}
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = Read<&K, A>> {
        self.entries.iter().map(move |(k, v)| Read {
            add_clock: self.clock.clone(),
            rm_clock: v.clock.clone(),
            value: k,
        })
    }

    pub fn values(&self) -> impl Iterator<Item = Read<&V, A>> {
        self.entries.values().map(move |v| Read {
            add_clock: self.clock.clone(),
            rm_clock: v.clock.clone(),
            value: &v.value,
        })
    }

    /// Gets an iterator over the entries of the `Map`.
    ///
    /// ```rust
    /// use libtheia::crdt::{ CmRDT, Map };
    /// use libtheia::crdt::multi_value::MultiValue;
    ///
    /// type Actor = &'static str;
    /// type Key = &'static str;
    ///
    /// let actor = "actor";
    ///
    /// let mut map: Map<i32, MultiValue<Key, Actor>, Actor> = Map::new();
    ///
    /// let add_a = map.read().derive_add(actor);
    /// map.apply(map.update(100, add_a, |v, a| v.write("foo", a)));
    ///
    /// let add_b = map.read().derive_add(actor);
    /// map.apply(map.update(50, add_b, |v, a| v.write("bar", a)));
    ///
    /// let add_c = map.read().derive_add(actor);
    /// map.apply(map.update(200, add_c, |v, a| v.write("baz", a)));
    ///
    ///
    /// let mut items: Vec<_> = map
    ///     .iterator()
    ///     .map(|item| (*item.value.0, item.value.1.read().value[0]))
    ///     .collect();
    ///
    /// items.sort();
    ///
    /// assert_eq!(items, &[(50, "bar"), (100, "foo"), (200, "baz")]);
    /// ```
    pub fn iterator(&self) -> impl Iterator<Item = Read<(&K, &V), A>> {
        self.entries.iter().map(move |(k, v)| Read {
            add_clock: self.clock.clone(),
            rm_clock: v.clock.clone(),
            value: (k, &v.value),
        })
    }
}
