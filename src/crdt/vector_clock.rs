//! Module containing Vector Clock implementation.
//!
//! ``` rust
//! use libtheia::crdt::CmRDT;
//! use libtheia::crdt::VectorClock;
//! use libtheia::crdt::Version;
//!
//! let mut a = VectorClock::new();
//! let mut b = VectorClock::new();
//! a.apply(Version::new("A", 2));
//! b.apply(Version::new("A", 1));
//! assert!(a > b);
//! ```

use core::cmp::{self, Ordering};
use core::convert::Infallible;
use core::fmt::{self, Debug, Display};
use core::mem;
use std::collections::{btree_map, BTreeMap};

use serde::{Deserialize, Serialize};
use crate::crdt::{Version, VersionRange, CmRDT, CvRDT, ResetRemove };

/// It contains a set of "actors" and associated counters.
/// When a particular actor witnesses a mutation, their associated
/// counter in a `VectorClock` is incremented, it tracks causality.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VectorClock<A: Ord> {
    pub versions: BTreeMap<A, u64>
}

impl<A: Ord> Default for VectorClock<A> {
    fn default() -> Self {
        Self {
            versions: BTreeMap::new(),
        }
    }
}

impl<A: Ord> PartialOrd for VectorClock<A> {
    fn partial_cmp(&self, other: &VectorClock<A>) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if other.versions.iter().all(|(w, c)| self.get(w) >= *c) {
            Some(Ordering::Greater)
        } else if self.versions.iter().all(|(w, c)| other.get(w) >= *c) {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}

impl<A: Ord + Display> Display for VectorClock<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<")?;
        for (i, (actor, count)) in self.versions.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}:{}", actor, count)?;
        }
        write!(f, ">")
    }
}

impl<A: Ord> ResetRemove<A> for VectorClock<A> {
    fn reset_remove(&mut self, other: &Self) {
        for Version { actor, counter } in other.iterator() {
            if counter >= self.get(actor) {
                self.versions.remove(actor);
            }
        }
    }
}

impl<A: Ord + Clone + Debug> CmRDT for VectorClock<A> {
    type Operation = Version<A>;
    type Validation = VersionRange<A>;

    fn validate(&self, version: &Self::Operation) -> Result<(), Self::Validation> {
        let next_counter = self.get(&version.actor) + 1;
        if version.counter > next_counter {
            Err(VersionRange {
                actor: version.actor.clone(),
                counter_range: next_counter..version.counter,
            })
        } else {
            Ok(())
        }
    }

    /// Monotonically adds the given actor version to this Vector Clock.
    ///
    /// ``` rust
    /// use libtheia::crdt::CmRDT;
    /// use libtheia::crdt::VectorClock;
    /// use libtheia::crdt::Version;
    ///
    /// let mut v = VectorClock::new();
    ///
    /// v.apply(Version::new("A", 2));
    ///
    /// // now all versions applied to `v` from actor `A` where
    /// // the counter is not bigger than 2 are nops.
    /// v.apply(Version::new("A", 0));
    /// assert_eq!(v.get(&"A"), 2);
    /// ```
    fn apply(&mut self, version: Self::Operation) {
        if self.get(&version.actor) < version.counter {
            self.versions.insert(version.actor, version.counter);
        }
    }
}

impl<A: Ord + Clone + Debug> CvRDT for VectorClock<A> {
    type Validation = Infallible;

    fn validate_merge(&self, _other: &Self) -> Result<(), Self::Validation> {
        Ok(())
    }

    fn merge(&mut self, other: Self) {
        for v in other.into_iter() {
            self.apply(v);
        }
    }
}

impl<A: Ord> VectorClock<A> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn clone_without(&self, base_clock: &VectorClock<A>) -> VectorClock<A> where A: Clone {
        let mut cloned = self.clone();
        cloned.reset_remove(base_clock);
        cloned
    }

    /// Generate Operation to increment an actor's counter.
    ///
    /// ``` rust
    /// use libtheia::crdt::CmRDT;
    /// use libtheia::crdt::VectorClock;
    ///
    /// let mut a = VectorClock::new();
    ///
    /// // `a.inc()` does not mutate the vclock!
    /// let op = a.increment("A");
    /// assert_eq!(a, VectorClock::new());
    ///
    /// // we must apply the op to the VClock to have
    /// // its edit take effect.
    /// a.apply(op.clone());
    /// assert_eq!(a.get(&"A"), 1);
    ///
    /// // Op's can be replicated to another node and
    /// // applied to the local state there.
    /// let mut other_node = VectorClock::new();
    /// other_node.apply(op);
    /// assert_eq!(other_node.get(&"A"), 1);
    /// ```
    pub fn increment(&self, actor: A) -> Version<A> where A: Clone {
        self.version(actor).inc()
    }

    pub fn get(&self, actor: &A) -> u64 {
        self.versions.get(actor).cloned().unwrap_or(0)
    }

    pub fn version(&self, actor: A) -> Version<A> {
        let counter = self.get(&actor);
        Version::new(actor, counter)
    }

    /// True if two vector clocks have diverged.
    ///
    /// ``` rust
    /// use libtheia::crdt::CmRDT;
    /// use libtheia::crdt::VectorClock;
    ///
    /// let (mut a, mut b) = (VectorClock::new(), VectorClock::new());
    /// a.apply(a.increment("A"));
    /// b.apply(b.increment("B"));
    /// assert!(a.concurrent(&b));
    /// ```
    pub fn concurrent(&self, other: &VectorClock<A>) -> bool {
        self.partial_cmp(other).is_none()
    }

    pub fn is_empty(&self) -> bool {
        self.versions.is_empty()
    }

    pub fn intersection(left: &VectorClock<A>, right: &VectorClock<A>) -> VectorClock<A> where A: Clone {
        let mut versions = BTreeMap::new();
        for (left_actor, left_counter) in left.versions.iter() {
            let right_counter = right.get(left_actor);
            if right_counter == *left_counter {
                versions.insert(left_actor.clone(), *left_counter);
            }
        }
        Self { versions }
    }

    /// Reduces this Vector Clock to the greatest-lower-bound
    ///
    /// ``` rust
    /// use libtheia::crdt::CmRDT;
    /// use libtheia::crdt::VectorClock;
    /// use libtheia::crdt::Version;
    ///
    /// let mut c = VectorClock::new();
    /// c.apply(Version::new(23, 6));
    /// c.apply(Version::new(89, 14));
    /// let c2 = c.clone();
    ///
    /// c.greatest_lower_bound(&c2);
    /// assert_eq!(c, c2);
    ///
    /// c.apply(Version::new(43, 1));
    /// assert_eq!(c.get(&43), 1);
    /// c.greatest_lower_bound(&c2);
    /// assert_eq!(c.get(&43), 0);
    /// ```
    pub fn greatest_lower_bound(&mut self, other: &Self) {
        self.versions = mem::take(&mut self.versions)
            .into_iter()
            .filter_map(|(actor, count)| {
                let min_count = cmp::min(count, other.get(&actor));
                match min_count {
                    0 => None,
                    _ => Some((actor, min_count)),
                }
            })
            .collect();
    }

    pub fn iterator(&self) -> impl Iterator<Item = Version<&A>> {
        self.versions.iter().map(|(a, c)| Version {
            actor: a,
            counter: *c,
        })
    }
}

pub struct IntoIter<A: Ord> {
    btree_iter: btree_map::IntoIter<A, u64>,
}

impl<A: Ord> std::iter::Iterator for IntoIter<A> {
    type Item = Version<A>;

    fn next(&mut self) -> Option<Version<A>> {
        self.btree_iter
            .next()
            .map(|(actor, counter)| Version::new(actor, counter))
    }
}

impl<A: Ord> std::iter::IntoIterator for VectorClock<A> {
    type Item = Version<A>;
    type IntoIter = IntoIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            btree_iter: self.versions.into_iter(),
        }
    }
}

impl<A: Ord + Clone + Debug> std::iter::FromIterator<Version<A>> for VectorClock<A> {
    fn from_iter<I: IntoIterator<Item = Version<A>>>(iter: I) -> Self {
        let mut clock = VectorClock::default();

        for v in iter {
            clock.apply(v);
        }

        clock
    }
}

impl<A: Ord + Clone + Debug> From<Version<A>> for VectorClock<A> {
    fn from(v: Version<A>) -> Self {
        let mut clock = VectorClock::default();
        clock.apply(v);
        clock
    }
}

