use std::error::Error;
use std::hash::Hash;
use crate::crdt::VectorClock;

pub trait Actor: Ord + Clone + Hash {}
impl<A: Ord + Clone + Hash> Actor for A {}

pub trait CvRDT {
    type Validation: Error;

    fn validate_merge(&self, other: &Self) -> Result<(), Self::Validation>;

    fn merge(&mut self, other: Self);
}


pub trait CmRDT {
    type Operation;

    type Validation: Error;

    fn validate_apply(&self, op: &Self::Operation) -> Result<(), Self::Validation>;

    fn apply(&mut self, op: Self::Operation);
}


pub trait Reset<A: Ord> {
    fn reset(&mut self, clock: &VectorClock<A>);
}