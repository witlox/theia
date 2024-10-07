use core::cmp::Ordering;
use core::fmt;
use num::{BigRational, One, Zero};
use serde::{Deserialize, Serialize};


fn rational_between(low: Option<&BigRational>, high: Option<&BigRational>) -> BigRational {
    match (low, high) {
        (None, None) => BigRational::zero(),
        (Some(low), None) => low + BigRational::one(),
        (None, Some(high)) => high - BigRational::one(),
        (Some(low), Some(high)) => (low + high) / BigRational::from_integer(2.into()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Identifier<T>(Vec<(BigRational, T)>);

impl<T: Ord> PartialOrd for Identifier<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord> Ord for Identifier<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut self_path = self.0.iter();
        let mut other_path = other.0.iter();
        loop {
            match (self_path.next(), other_path.next()) {
                (Some(self_node), Some(other_node)) => match self_node.cmp(other_node) {
                    Ordering::Equal => continue,
                    ord => return ord,
                },
                (None, Some(_)) => return Ordering::Greater,
                (Some(_), None) => return Ordering::Less,
                (None, None) => return Ordering::Equal,
            }
        }
    }
}

impl<T> From<(BigRational, T)> for Identifier<T> {
    fn from((rational, value): (BigRational, T)) -> Self {
        Self(vec![(rational, value)])
    }
}

impl<T: Clone + Ord + Eq> Identifier<T> {
    pub fn value(&self) -> &T {
        self.0.last().map(|(_, e)| e).unwrap()
    }

    pub fn into_value(mut self) -> T {
        self.0.pop().map(|(_, e)| e).unwrap()
    }

    pub fn between(low: Option<&Self>, high: Option<&Self>, cursor: T) -> Self {
        match (low, high) {
            (Some(low), Some(high)) => {
                match low.cmp(high) {
                    Ordering::Greater => return Self::between(Some(high), Some(low), cursor),
                    Ordering::Equal => return high.clone(),
                    _ => (),
                }

                let mut path: Vec<(BigRational, T)> = vec![];

                let mut low_path: Box<dyn Iterator<Item = &(BigRational, T)>> =
                    Box::new(low.0.iter());
                let mut high_path: Box<dyn Iterator<Item = &(BigRational, T)>> =
                    Box::new(high.0.iter());
                loop {
                    match (low_path.next(), high_path.next()) {
                        (Some((lr, l_cursor)), Some((hr, h_cursor))) if lr == hr => {
                            if l_cursor < &cursor && &cursor < h_cursor {
                                path.push((hr.clone(), cursor));
                                break;
                            } else if l_cursor == h_cursor {
                                path.push((hr.clone(), h_cursor.clone()));
                            } else {
                                path.push((hr.clone(), h_cursor.clone()));
                                low_path = Box::new(std::iter::empty());
                            }
                        }
                        (low_node, high_node) => {
                            path.push((
                                rational_between(low_node.map(|n| &n.0), high_node.map(|n| &n.0)),
                                cursor,
                            ));
                            break;
                        }
                    }
                }
                Self(path)
            }

            (low, high) => Self(vec![(
                rational_between(
                    low.and_then(|low_entry| low_entry.0.first().map(|(r, _)| r)),
                    high.and_then(|high_entry| high_entry.0.first().map(|(r, _)| r)),
                ),
                cursor,
            )]),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Identifier<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ID[")?;
        let mut iter = self.0.iter();
        if let Some((r, e)) = iter.next() {
            write!(f, "{}:{}", r, e)?;
        }
        for (r, e) in iter {
            write!(f, ", {}:{}", r, e)?;
        }
        write!(f, "]")
    }
}