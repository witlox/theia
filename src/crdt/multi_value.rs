use core::cmp::Ordering;
use core::convert::Infallible;
use core::fmt::{self, Debug, Display};
use core::mem;

use serde::{Deserialize, Serialize};
use crate::crdt::base::{Add, Read};
use crate::crdt::traits::{CmRDT, CvRDT, Reset};
use crate::crdt::vector_clock::VectorClock;

/// Multi-Value storage
///
/// ``` rust
/// use libtheia::crdt::{ CmRDT, Version, VectorClock };
/// use libtheia::crdt::multi_value::MultiValue;
///
/// let mut r1 = MultiValue::new();
/// let mut r2 = r1.clone();
/// let r1_read_ctx = r1.read();
/// let r2_read_ctx = r2.read();
///
/// r1.apply(r1.write("foo", r1_read_ctx.derive_add(123)));
///
/// let op = r2.write("bar", r2_read_ctx.derive_add(111));
/// r2.apply(op.clone());
///
/// r1.apply(op);
///
/// assert_eq!(r1.read().value, vec!["foo", "bar"]);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MultiValue<V, A: Ord> {
    values: Vec<(VectorClock<A>, V)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation<V, A: Ord> {
    Put {
        clock: VectorClock<A>,
        value: V,
    },
}

impl<V: Display, A: Ord + Display> Display for MultiValue<V, A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "|")?;
        for (i, (ctx, val)) in self.values.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}@{}", val, ctx)?;
        }
        write!(f, "|")
    }
}

impl<V: PartialEq, A: Ord> PartialEq for MultiValue<V, A> {
    fn eq(&self, other: &Self) -> bool {
        for v in self.values.iter() {
            let num_found = other.values.iter().filter(|d| d == &v).count();
            if num_found == 0 {
                return false;
            }
            assert_eq!(num_found, 1);
        }
        for v in other.values.iter() {
            let num_found = self.values.iter().filter(|d| d == &v).count();
            if num_found == 0 {
                return false;
            }
            assert_eq!(num_found, 1);
        }
        true
    }
}

impl<V: Eq, A: Ord> Eq for MultiValue<V, A> {}

impl<V, A: Ord> Reset<A> for MultiValue<V, A> {
    fn reset(&mut self, clock: &VectorClock<A>) {
        self.values = mem::take(&mut self.values)
            .into_iter()
            .filter_map(|(mut val_clock, val)| {
                val_clock.reset(clock);
                if val_clock.is_empty() {
                    None // remove this value from the register
                } else {
                    Some((val_clock, val))
                }
            })
            .collect()
    }
}

impl<V, A: Ord> Default for MultiValue<V, A> {
    fn default() -> Self {
        Self { values: Vec::new() }
    }
}

impl<V, A: Ord> CvRDT for MultiValue<V, A> {
    type Validation = Infallible;

    fn validate_merge(&self, _other: &Self) -> Result<(), Self::Validation> {
        Ok(())
    }

    fn merge(&mut self, other: Self) {
        self.values = mem::take(&mut self.values)
            .into_iter()
            .filter(|(clock, _)| other.values.iter().filter(|(c, _)| clock < c).count() == 0)
            .collect();

        self.values.extend(
            other
                .values
                .into_iter()
                .filter(|(clock, _)| self.values.iter().filter(|(c, _)| clock < c).count() == 0)
                .filter(|(clock, _)| self.values.iter().all(|(c, _)| clock != c))
                .collect::<Vec<_>>(),
        );
    }
}

impl<V, A: Ord> CmRDT for MultiValue<V, A> {
    type Operation = Operation<V, A>;
    type Validation = Infallible;

    fn validate_apply(&self, _operation: &Self::Operation) -> Result<(), Self::Validation> {
        Ok(())
    }

    fn apply(&mut self, operation: Self::Operation) {
        match operation {
            Operation::Put { clock, value: val } => {
                if clock.is_empty() {
                    return;
                }
                self.values.retain(|(val_clock, _)| {
                    matches!(
                        val_clock.partial_cmp(&clock),
                        None | Some(Ordering::Greater)
                    )
                });
                let mut should_add = true;
                for (existing_clock, _) in self.values.iter() {
                    if existing_clock > &clock {
                        should_add = false;
                    }
                }
                if should_add {
                    self.values.push((clock, val));
                }
            }
        }
    }
}

impl<V, A: Ord + Clone + Debug> MultiValue<V, A> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn write(&self, value: V, a: Add<A>) -> Operation<V, A> {
        Operation::Put {
            clock: a.clock,
            value,
        }
    }

    pub fn read(&self) -> Read<Vec<V>, A> where V: Clone {
        let clock = self.clock();
        let concurrent_vals = self.values.iter().cloned().map(|(_, v)| v).collect();

        Read {
            add_clock: clock.clone(),
            remove_clock: clock,
            value: concurrent_vals,
        }
    }

    pub fn read_all(&self) -> Read<(), A> {
        let clock = self.clock();
        Read {
            add_clock: clock.clone(),
            remove_clock: clock,
            value: (),
        }
    }

    fn clock(&self) -> VectorClock<A> {
        self.values
            .iter()
            .fold(VectorClock::new(), |mut accum_clock, (c, _)| {
                accum_clock.merge(c.clone());
                accum_clock
            })
    }
}