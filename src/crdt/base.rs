use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use crate::crdt::vector_clock::VectorClock;
use crate::crdt::version::Version;

use crate::crdt::traits::CmRDT;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Read<V, A: Ord> {
    pub add_clock: VectorClock<A>,
    pub rm_clock: VectorClock<A>,
    pub value: V,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Add<A: Ord> {
    pub clock: VectorClock<A>,
    pub version: Version<A>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remove<A: Ord> {
    pub clock: VectorClock<A>,
}

impl<V, A: Ord + Clone + Debug> Read<V, A> {
    pub fn derive_add(self, version: A) -> Add<A> {
        let mut clock = self.add_clock;
        let v = clock.increment(version);
        clock.apply(v.clone());
        Add { clock, version: v }
    }

    pub fn derive_remove(self) -> Remove<A> {
        Remove {
            clock: self.rm_clock,
        }
    }

    pub fn split(self) -> (V, Read<(), A>) {
        (
            self.value,
            Read {
                add_clock: self.add_clock,
                rm_clock: self.rm_clock,
                value: (),
            },
        )
    }
}