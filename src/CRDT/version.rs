use std::cmp::{Ordering, PartialOrd};
use std::fmt;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Version<A> {
    pub actor: A,
    pub counter: u64,
}

impl<A> Version<A> {
    pub fn new(actor: A, counter: u64) -> Self {
        Self { actor, counter }
    }

    pub fn apply_increment(&mut self) {
        self.counter += 1;
    }
}

impl<A: Clone> Version<A> {
    pub fn inc(&self) -> Self {
        Self {
            actor: self.actor.clone(),
            counter: self.counter + 1,
        }
    }
}
impl<A: Copy> Copy for Version<A> {}

impl<A: PartialEq> PartialEq for Version<A> {
    fn eq(&self, other: &Self) -> bool {
        self.actor == other.actor && self.counter == other.counter
    }
}

impl<A: Eq> Eq for Version<A> {}

impl<A: Hash> Hash for Version<A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.actor.hash(state);
        self.counter.hash(state);
    }
}

impl<A: PartialOrd> PartialOrd for Version<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.actor == other.actor {
            self.counter.partial_cmp(&other.counter)
        } else {
            None
        }
    }
}

impl<A: fmt::Debug> fmt::Debug for Version<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}.{:?}", self.actor, self.counter)
    }
}

impl<A> From<(A, u64)> for Version<A> {
    fn from(dot_material: (A, u64)) -> Self {
        let (actor, counter) = dot_material;
        Self { actor, counter }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct OrderedVersion<A: Ord> {
    pub actor: A,
    pub counter: u64,
}

impl<A: Ord> From<OrderedVersion<A>> for Version<A> {
    fn from(OrderedVersion { actor, counter }: OrderedVersion<A>) -> Self {
        Self { actor, counter }
    }
}

impl<A: Ord> From<Version<A>> for OrderedVersion<A> {
    fn from(Version { actor, counter }: Version<A>) -> Self {
        Self { actor, counter }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct VersionRange<A> {
    pub actor: A,
    pub counter_range: core::ops::Range<u64>,
}

impl<A: fmt::Debug + Ord> fmt::Display for OrderedVersion<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}.{}", self.actor, self.counter)
    }
}

impl<A: fmt::Debug> fmt::Display for VersionRange<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}.({}..{})",
            self.actor, self.counter_range.start, self.counter_range.end
        )
    }
}

impl<A: fmt::Debug> std::error::Error for VersionRange<A> {}